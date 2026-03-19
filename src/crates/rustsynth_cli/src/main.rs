use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};

use rustsynth_eisenscript::parser;
use rustsynth_eisenscript::preprocessor;
use rustsynth_eval::{build, BuildConfig, RecursionMode};
use rustsynth_semantics::{resolve, validate};

// ─────────────────────────────────────────────────────────────────────────────
// CLI definition
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "rustsynth", about = "EisenScript compiler and scene generator")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Compile an EisenScript file and print the scene as JSON to stdout.
    Build {
        /// Path to the EisenScript source file.
        file: PathBuf,
        #[command(flatten)]
        opts: CommonOpts,
    },
    /// Compile and export the scene as OBJ (writes .obj and .mtl files).
    ExportObj {
        /// Path to the EisenScript source file.
        file: PathBuf,
        /// Output .obj file path.
        #[arg(short, long)]
        output: PathBuf,
        #[command(flatten)]
        opts: CommonOpts,
    },
    /// Compile and export the scene via a template.
    ExportTemplate {
        /// Path to the EisenScript source file.
        file: PathBuf,
        /// Output file path.
        #[arg(short, long)]
        output: PathBuf,
        /// Path to a custom template XML file (uses a built-in text dump if omitted).
        #[arg(long)]
        template: Option<PathBuf>,
        #[command(flatten)]
        opts: CommonOpts,
    },
}

#[derive(Parser)]
struct CommonOpts {
    /// RNG seed.
    #[arg(long, default_value_t = 0)]
    seed: u64,
    /// Maximum number of emitted objects.
    #[arg(long, default_value_t = 100_000)]
    max_objects: usize,
    /// Maximum BFS/DFS generations.
    #[arg(long, default_value_t = 1000)]
    max_generations: u32,
    /// Recursion mode.
    #[arg(long, value_enum, default_value_t = CliMode::Bfs)]
    mode: CliMode,
}

#[derive(Clone, ValueEnum)]
enum CliMode {
    Bfs,
    Dfs,
}

// ─────────────────────────────────────────────────────────────────────────────
// Pipeline (mirrors rustsynth_app_tauri::pipeline)
// ─────────────────────────────────────────────────────────────────────────────

fn run_pipeline(
    source: &str,
    config: &BuildConfig,
) -> Result<rustsynth_scene::Scene> {
    use rustsynth_eisenscript::diagnostics::Severity;

    // 1. Preprocess
    let pre = preprocessor::preprocess(source, config.seed);
    for d in &pre.diagnostics {
        let label = if matches!(d.severity, Severity::Error) { "ERROR" } else { "WARN" };
        eprintln!("[{}] line {}: {}", label, d.line, d.message);
    }

    // 2. Parse
    let parse = parser::parse(&pre.output);
    for d in &parse.diagnostics {
        let label = if matches!(d.severity, Severity::Error) { "ERROR" } else { "WARN" };
        eprintln!("[{}] line {}: {}", label, d.line, d.message);
    }

    // 3. Resolve
    let (graph, resolve_diags) = resolve(&parse.script);
    for d in &resolve_diags {
        let label = if matches!(d.severity, Severity::Error) { "ERROR" } else { "WARN" };
        eprintln!("[{}] line {}: {}", label, d.line, d.message);
    }

    // 4. Validate
    let val_diags = validate(&graph);
    for d in &val_diags {
        let label = if matches!(d.severity, Severity::Error) { "ERROR" } else { "WARN" };
        eprintln!("[{}] line {}: {}", label, d.line, d.message);
    }

    // 5. Build
    let scene = build(&graph, config);
    Ok(scene)
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn build_config(opts: &CommonOpts) -> BuildConfig {
    BuildConfig {
        seed: opts.seed,
        max_objects: opts.max_objects,
        max_generations: opts.max_generations,
        mode: match opts.mode {
            CliMode::Bfs => RecursionMode::BreadthFirst,
            CliMode::Dfs => RecursionMode::DepthFirst,
        },
        ..Default::default()
    }
}

fn read_source(path: &PathBuf) -> Result<String> {
    std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))
}

// ─────────────────────────────────────────────────────────────────────────────
// Entry point
// ─────────────────────────────────────────────────────────────────────────────

fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Command::Build { file, opts } => {
            let source = read_source(&file)?;
            let config = build_config(&opts);
            let scene = run_pipeline(&source, &config)?;
            let json = serde_json::to_string_pretty(&scene)?;
            println!("{json}");
        }
        Command::ExportObj { file, output, opts } => {
            let source = read_source(&file)?;
            let config = build_config(&opts);
            let scene = run_pipeline(&source, &config)?;

            let exporter = rustsynth_export_obj::ObjExporter::default();
            let obj_output = exporter.export(&scene)?;

            std::fs::write(&output, &obj_output.obj)
                .with_context(|| format!("failed to write {}", output.display()))?;

            let mtl_path = output.with_extension("mtl");
            std::fs::write(&mtl_path, &obj_output.mtl)
                .with_context(|| format!("failed to write {}", mtl_path.display()))?;

            eprintln!("Wrote {} and {}", output.display(), mtl_path.display());
        }
        Command::ExportTemplate {
            file,
            output,
            template,
            opts,
        } => {
            use rustsynth_export_template::{Template, TemplateExporter};

            const BUILTIN_TEMPLATE_XML: &str = r#"<template name="TextDump"
        defaultExtension="Text file (*.txt)">
        <primitive name="begin">// RustSynth scene export\n// Objects:\n</primitive>
        <primitive name="box">box {matrix}\n</primitive>
        <primitive name="sphere">sphere cx={cx} cy={cy} cz={cz} r={rad} rgb={r},{g},{b}\n</primitive>
        <primitive name="cylinder">cylinder {matrix}\n</primitive>
        <primitive name="line">line {x1},{y1},{z1} {x2},{y2},{z2}\n</primitive>
        <primitive name="dot">dot {x},{y},{z}\n</primitive>
        <primitive name="grid">grid {matrix}\n</primitive>
        <primitive name="end">// end\n</primitive>
    </template>"#;

            let source = read_source(&file)?;
            let config = build_config(&opts);
            let scene = run_pipeline(&source, &config)?;

            let template_xml = match template {
                Some(ref p) => std::fs::read_to_string(p)
                    .with_context(|| format!("failed to read template {}", p.display()))?,
                None => BUILTIN_TEMPLATE_XML.to_string(),
            };

            let tmpl = Template::from_xml(&template_xml)?;
            let mut exporter = TemplateExporter::new(tmpl);
            let text = exporter.export(&scene)?;

            std::fs::write(&output, &text)
                .with_context(|| format!("failed to write {}", output.display()))?;

            eprintln!("Wrote {}", output.display());
        }
    }

    Ok(())
}
