//! EisenScript preprocessor.
//!
//! Placeholder skeleton — full implementation in T04.
//!
//! The real preprocessor handles:
//! - `#define name value` substitutions
//! - `#define name value (float:lo-hi)` GUI float parameters
//! - `#define name value (int:lo-hi)` GUI int parameters
//! - `random[lo,hi]` replacement using seeded RNG

use std::collections::BTreeMap;

use crate::diagnostics::Diagnostic;
use rustsynth_core::rng::Rng;

/// A GUI-controllable parameter extracted from preprocessor directives.
#[derive(Debug, Clone, PartialEq)]
pub enum GuiParam {
    Float { name: String, default: f64, min: f64, max: f64 },
    Int   { name: String, default: i64, min: i64, max: i64 },
}

/// Result of preprocessing.
#[derive(Debug, Clone)]
pub struct PreprocessResult {
    pub output: String,
    pub gui_params: Vec<GuiParam>,
    pub diagnostics: Vec<Diagnostic>,
}

/// Preprocess EisenScript source text.
///
/// `seed` is used for `random[lo,hi]` substitutions.
pub fn preprocess(source: &str, seed: u64) -> PreprocessResult {
    let mut rng = Rng::new(seed);
    let mut gui_params = Vec::new();
    let mut diagnostics = Vec::new();
    let mut substitutions = BTreeMap::<String, String>::new();
    let mut output_lines = Vec::new();

    for (index, line) in normalize_newlines(source).lines().enumerate() {
        let line_number = index + 1;

        if line.starts_with('#') {
            match parse_define(line) {
                Some(DefineDirective::Plain { name, value }) => {
                    if value.contains(&name) {
                        diagnostics.push(Diagnostic::warning(
                            line_number,
                            format!("#define command is recursive - skipped: {name} -> {value}"),
                        ));
                    } else {
                        substitutions.insert(name, value);
                    }
                    output_lines.push(String::new());
                }
                Some(DefineDirective::GuiFloat {
                    name,
                    value_text,
                    default,
                    min,
                    max,
                }) => {
                    if value_text.contains(&name) {
                        diagnostics.push(Diagnostic::warning(
                            line_number,
                            format!("#define command is recursive - skipped: {name} -> {value_text}"),
                        ));
                    } else {
                        gui_params.push(GuiParam::Float {
                            name: name.clone(),
                            default,
                            min,
                            max,
                        });
                        substitutions.insert(name, value_text);
                    }
                    output_lines.push(String::new());
                }
                Some(DefineDirective::GuiInt {
                    name,
                    value_text,
                    default,
                    min,
                    max,
                }) => {
                    if value_text.contains(&name) {
                        diagnostics.push(Diagnostic::warning(
                            line_number,
                            format!("#define command is recursive - skipped: {name} -> {value_text}"),
                        ));
                    } else {
                        gui_params.push(GuiParam::Int {
                            name: name.clone(),
                            default,
                            min,
                            max,
                        });
                        substitutions.insert(name, value_text);
                    }
                    output_lines.push(String::new());
                }
                None => {
                    diagnostics.push(Diagnostic::warning(
                        line_number,
                        format!("Could not understand preprocessor command: {line}"),
                    ));
                    output_lines.push(String::new());
                }
            }
        } else {
            let substituted = apply_substitutions(line, &substitutions, line_number, &mut diagnostics);
            output_lines.push(expand_random_calls(&substituted, &mut rng));
        }
    }

    PreprocessResult {
        output: output_lines.join("\n"),
        gui_params,
        diagnostics,
    }
}

fn normalize_newlines(source: &str) -> String {
    source.replace("\r\n", "\n").replace('\r', "\n")
}

fn apply_substitutions(
    input: &str,
    substitutions: &BTreeMap<String, String>,
    line_number: usize,
    diagnostics: &mut Vec<Diagnostic>,
) -> String {
    let mut output = input.to_owned();
    let mut substitution_rounds = 0usize;

    loop {
        if substitution_rounds > 100 {
            diagnostics.push(Diagnostic::warning(
                line_number,
                "More than 100 recursive preprocessor substitutions... breaking.",
            ));
            break;
        }

        let mut matched = false;
        for (name, value) in substitutions {
            if output.contains(name) {
                output = output.replace(name, value);
                substitution_rounds += 1;
                matched = true;
                break;
            }
        }

        if !matched {
            break;
        }
    }

    output
}

fn expand_random_calls(input: &str, rng: &mut Rng) -> String {
    let mut output = input.to_owned();
    let mut search_from = 0usize;

    while let Some(relative_start) = output[search_from..].find("random[") {
        let start = search_from + relative_start;
        let args_start = start + "random[".len();
        let Some(relative_end) = output[args_start..].find(']') else {
            break;
        };
        let end = args_start + relative_end;
        let args = &output[args_start..end];

        let Some((lo_text, hi_text)) = args.split_once(',') else {
            search_from = end + 1;
            continue;
        };

        let Ok(lo) = lo_text.parse::<f64>() else {
            search_from = end + 1;
            continue;
        };
        let Ok(hi) = hi_text.parse::<f64>() else {
            search_from = end + 1;
            continue;
        };

        let replacement = rng.next_range_f64(lo, hi).to_string();
        output.replace_range(start..=end, &replacement);
        search_from = start + replacement.len();
    }

    output
}

enum DefineDirective {
    Plain {
        name: String,
        value: String,
    },
    GuiFloat {
        name: String,
        value_text: String,
        default: f64,
        min: f64,
        max: f64,
    },
    GuiInt {
        name: String,
        value_text: String,
        default: i64,
        min: i64,
        max: i64,
    },
}

fn parse_define(line: &str) -> Option<DefineDirective> {
    let rest = line.strip_prefix("#define")?.trim_start();
    let (name, remainder) = split_name_and_value(rest)?;

    if let Some((value_text, range_text)) = parse_gui_suffix(remainder, "float") {
        let (min_text, max_text) = parse_range(range_text)?;
        let default = value_text.parse::<f64>().ok()?;
        let min = min_text.parse::<f64>().ok()?;
        let max = max_text.parse::<f64>().ok()?;
        return Some(DefineDirective::GuiFloat {
            name: name.to_owned(),
            value_text: value_text.to_owned(),
            default,
            min,
            max,
        });
    }

    if let Some((value_text, range_text)) = parse_gui_suffix(remainder, "int") {
        let (min_text, max_text) = parse_range(range_text)?;
        let default = value_text.parse::<i64>().ok()?;
        let min = min_text.parse::<i64>().ok()?;
        let max = max_text.parse::<i64>().ok()?;
        return Some(DefineDirective::GuiInt {
            name: name.to_owned(),
            value_text: value_text.to_owned(),
            default,
            min,
            max,
        });
    }

    Some(DefineDirective::Plain {
        name: name.to_owned(),
        value: remainder.trim().to_owned(),
    })
}

fn split_name_and_value(rest: &str) -> Option<(&str, &str)> {
    let first_space = rest.find(char::is_whitespace)?;
    let name = &rest[..first_space];
    let value = rest[first_space..].trim();

    if name.is_empty() || value.is_empty() {
        return None;
    }

    Some((name, value))
}

fn parse_gui_suffix<'a>(value: &'a str, kind: &str) -> Option<(&'a str, &'a str)> {
    let prefix = format!("({kind}:");
    let without_close = value.strip_suffix(')')?;
    let (value_part, range_part) = without_close.rsplit_once(&prefix)?;
    let value_text = value_part.trim_end();
    let range_text = range_part.trim();

    if value_text.is_empty() || range_text.is_empty() {
        return None;
    }

    Some((value_text, range_text))
}

fn parse_range(range: &str) -> Option<(&str, &str)> {
    let pieces: Vec<_> = range.split('-').collect();
    if pieces.len() != 2 {
        return None;
    }

    let min = pieces[0].trim();
    let max = pieces[1].trim();
    if min.is_empty() || max.is_empty() {
        return None;
    }

    Some((min, max))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    const PREPROCESSOR_FIXTURE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/eisenscript/Tutorials/Preprocessor.es"
    ));
    const PREPROCESSOR_GUI_FIXTURE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/eisenscript/Tutorials/PreprocessorGUI.es"
    ));

    #[test]
    fn expands_plain_define_fixture() {
        let result = preprocess(PREPROCESSOR_FIXTURE, 0);

        assert!(result.diagnostics.is_empty());
        assert!(result
            .output
            .lines()
            .all(|line| !line.trim_start().starts_with("#define")));
        assert!(result.output.contains("rz 6 ry 6 s 0.98"));
        assert!(result.output.contains("rz 6 ry 6 s 0.98 hue 1  sat 1 a 0.99"));
        assert!(result.gui_params.is_empty());
    }

    #[test]
    fn extracts_gui_parameters_and_applies_default_values() {
        let result = preprocess(PREPROCESSOR_GUI_FIXTURE, 0);

        assert!(result.diagnostics.is_empty());
        assert!(result
            .output
            .lines()
            .all(|line| !line.trim_start().starts_with("#define")));
        assert!(result.output.contains("6 * { rx 10  x 0.2 sat 0.95  } R"));
        assert!(result.output.contains("ry 20 s 0.94 hue 1 a 0.99"));
        assert!(result.output.contains("rz 6 ry 0 s 0.94 hue 1 a 0.99"));
        assert_eq!(
            result.gui_params,
            vec![
                GuiParam::Float {
                    name: "sizeStep".to_owned(),
                    default: 0.94,
                    min: 0.0,
                    max: 1.0,
                },
                GuiParam::Float {
                    name: "angle1".to_owned(),
                    default: 20.0,
                    min: 0.0,
                    max: 90.0,
                },
                GuiParam::Float {
                    name: "angle2".to_owned(),
                    default: 6.0,
                    min: 0.0,
                    max: 90.0,
                },
                GuiParam::Int {
                    name: "iterations".to_owned(),
                    default: 6,
                    min: 1,
                    max: 90,
                },
            ]
        );
    }

    #[test]
    fn warns_and_skips_recursive_defines() {
        let result = preprocess("#define foo foo + 1\nfoo", 0);

        assert_eq!(result.output.trim(), "foo");
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].line, 1);
        assert!(result.diagnostics[0].message.contains("recursive"));
    }

    #[test]
    fn expands_random_calls_deterministically() {
        let left = preprocess("value random[-2,3] random[0,1]", 42);
        let right = preprocess("value random[-2,3] random[0,1]", 42);

        assert_eq!(left.output, right.output);
        assert!(!left.output.contains("random["));

        let numbers: Vec<f64> = left
            .output
            .split_whitespace()
            .skip(1)
            .map(|value| value.parse::<f64>().unwrap())
            .collect();
        assert_eq!(numbers.len(), 2);
        assert!((-2.0..3.0).contains(&numbers[0]));
        assert!((0.0..1.0).contains(&numbers[1]));
    }
}
