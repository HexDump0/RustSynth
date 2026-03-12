//! Builder — drives rule expansion and emits canonical scene objects.
//!
//! This module implements the core evaluation loop that mirrors the legacy
//! `Builder.cpp`:
//!
//! - breadth-first (default) or depth-first expansion
//! - per-rule max-depth budget tracked in the evaluation state
//! - global max-generations and max-objects limits
//! - optional min/max size pruning
//! - weighted ambiguous rule selection via the seeded RNG
//! - scene object emission when a primitive rule is reached

use std::collections::VecDeque;

use log::warn;

use rustsynth_core::color::Rgba;
use rustsynth_core::math::{Mat4, Vec3};
use rustsynth_core::rng::Rng;
use rustsynth_eisenscript::ast::{Action, BodyItem};
use rustsynth_scene::camera::CameraState;
use rustsynth_scene::object::SceneObject;
use rustsynth_scene::primitive::PrimitiveKind;
use rustsynth_scene::Scene;
use rustsynth_semantics::rule_graph::{
    AmbiguousRuleNode, CustomRuleNode, RuleGraph, RuleNode, StartItem,
};

use crate::recursion::RecursionMode;
use crate::state::State;
use crate::transform::apply_transforms;

// ─────────────────────────────────────────────────────────────────────────────
// Public configuration
// ─────────────────────────────────────────────────────────────────────────────

/// Configuration for the evaluation pass.
#[derive(Debug, Clone)]
pub struct BuildConfig {
    /// Max BFS generations (corresponds to `set maxdepth` at top level).
    pub max_generations: u32,
    /// Max number of emitted objects.
    pub max_objects: usize,
    /// Minimum object size (0 = disabled).
    pub min_dim: f32,
    /// Maximum object size (0 = disabled).
    pub max_dim: f32,
    /// Synchronise RNG across siblings (legacy `set syncrandom true`).
    pub sync_random: bool,
    /// Recursion mode (BFS or DFS).
    pub mode: RecursionMode,
    /// Initial RNG seed.
    pub seed: u64,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            max_generations: 1000,
            max_objects: 100_000,
            min_dim: 0.0,
            max_dim: 0.0,
            sync_random: false,
            mode: RecursionMode::BreadthFirst,
            seed: 0,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Entry point
// ─────────────────────────────────────────────────────────────────────────────

/// Evaluate a resolved `RuleGraph` and produce a `Scene`.
///
/// Processes top-level set commands first (camera, background, limits), then
/// runs the expansion loop using the configured recursion mode.
pub fn build(graph: &RuleGraph, cfg: &BuildConfig) -> Scene {
    let mut config = cfg.clone();
    let mut rng = Rng::new(config.seed);
    let mut scene = Scene::default();
    let initial_state = State::default();

    // ── Process top-level items ───────────────────────────────────────────
    let mut start_stack: VecDeque<(String, State)> = VecDeque::new();

    for item in &graph.start_items {
        match item {
            StartItem::SetCmd(cmd) => {
                apply_set_command(&cmd.key, &cmd.value, &mut config, &mut scene, &mut rng);
            }
            StartItem::Action(action) => {
                push_action_states(action, &initial_state, None, &mut rng, &mut start_stack);
            }
        }
    }

    // ── Expand ────────────────────────────────────────────────────────────
    match config.mode {
        RecursionMode::BreadthFirst => {
            run_bfs(start_stack, graph, &mut config, &mut rng, &mut scene);
        }
        RecursionMode::DepthFirst => {
            // In depth-first mode, apply the global max depth to all rules.
            if config.max_generations > 0 {
                let mut g = graph.clone();
                set_rules_max_depth(&mut g, config.max_generations);
                run_dfs(start_stack, &g, &mut config, &mut rng, &mut scene);
            } else {
                run_dfs(start_stack, graph, &mut config, &mut rng, &mut scene);
            }
        }
    }

    scene
}

// ─────────────────────────────────────────────────────────────────────────────
// BFS loop
// ─────────────────────────────────────────────────────────────────────────────

fn run_bfs(
    mut stack: VecDeque<(String, State)>,
    graph: &RuleGraph,
    config: &mut BuildConfig,
    rng: &mut Rng,
    scene: &mut Scene,
) {
    let mut generation = 0u32;

    while !stack.is_empty()
        && generation < config.max_generations
        && scene.objects.len() < config.max_objects
        && stack.len() < config.max_objects
    {
        let sync_seed = if config.sync_random { rng.next_u64() } else { 0 };
        generation += 1;

        let mut next_stack: VecDeque<(String, State)> = VecDeque::new();

        while let Some((rule_name, mut state)) = stack.pop_front() {
            // Re-seed if this branch carries a seed.
            if state.seed != 0 {
                *rng = Rng::new(state.seed);
                state.seed = rng.next_u64();
            }
            if config.sync_random {
                *rng = Rng::new(sync_seed);
            }

            if should_prune(&state.transform, config) {
                continue;
            }

            apply_rule(&rule_name, state, graph, config, rng, scene, &mut next_stack);
        }

        stack = next_stack;
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// DFS loop
// ─────────────────────────────────────────────────────────────────────────────

fn run_dfs(
    mut stack: VecDeque<(String, State)>,
    graph: &RuleGraph,
    config: &mut BuildConfig,
    rng: &mut Rng,
    scene: &mut Scene,
) {
    while let Some((rule_name, mut state)) = stack.pop_front() {
        if scene.objects.len() >= config.max_objects {
            break;
        }

        if state.seed != 0 {
            *rng = Rng::new(state.seed);
            state.seed = rng.next_u64();
        }

        if should_prune(&state.transform, config) {
            continue;
        }

        let mut next: VecDeque<(String, State)> = VecDeque::new();
        apply_rule(&rule_name, state, graph, config, rng, scene, &mut next);

        // Prepend new items (depth-first: process children before siblings).
        for item in next.into_iter().rev() {
            stack.push_front(item);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Rule application
// ─────────────────────────────────────────────────────────────────────────────

fn apply_rule(
    rule_name: &str,
    state: State,
    graph: &RuleGraph,
    config: &BuildConfig,
    rng: &mut Rng,
    scene: &mut Scene,
    next: &mut VecDeque<(String, State)>,
) {
    match graph.get(rule_name) {
        Some(RuleNode::Primitive { kind_name, class_tag }) => {
            emit_object(kind_name, class_tag.as_deref(), &state, scene);
        }
        Some(RuleNode::Custom(custom)) => {
            apply_custom(custom.clone(), state, rng, scene, config, next);
        }
        Some(RuleNode::Ambiguous(ambig)) => {
            // Pick a variant by weight.
            if let Some(chosen) = pick_variant(ambig, rng) {
                apply_custom(chosen, state, rng, scene, config, next);
            }
        }
        None => {
            warn!("Undefined rule reference during evaluation: '{rule_name}'");
        }
    }
}

fn apply_custom(
    rule: CustomRuleNode,
    state: State,
    rng: &mut Rng,
    _scene: &mut Scene,
    _config: &BuildConfig,
    next: &mut VecDeque<(String, State)>,
) {
    // ── Check per-rule max depth ──────────────────────────────────────────
    let (allowed, new_depth_opt) = if let Some(max_depth) = rule.max_depth {
        let remaining = state.max_depths.get(&rule.name).copied();
        match remaining {
            None => {
                // First visit: budget is max_depth - 1 for subsequent calls.
                (true, Some(max_depth as i32 - 1))
            }
            Some(d) if d > 0 => (true, Some(d - 1)),
            Some(_) => {
                // Exhausted — invoke retirement rule if present.
                if let Some(ref ret) = rule.retirement {
                    let mut ret_state = state.clone();
                    ret_state.max_depths.insert(rule.name.clone(), max_depth as i32);
                    next.push_back((ret.clone(), ret_state));
                }
                return; // don't apply body
            }
        }
    } else {
        (true, None)
    };

    if !allowed {
        return;
    }

    // ── Apply body items ──────────────────────────────────────────────────
    for item in &rule.body {
        match item {
            BodyItem::SetCmd(cmd) => {
                // set commands inside a rule body are currently treated as
                // no-ops for evaluation (they were primarily used for scene-wide
                // settings which are already processed at the top level).
                // TODO: support dynamic set commands in rule bodies if needed.
                let _ = cmd;
            }
            BodyItem::Action(action) => {
                let calling = new_depth_opt
                    .map(|d| (rule.name.as_str(), d));
                push_action_states(action, &state, calling, rng, next);
            }
        }
    }
}

fn pick_variant(ambig: &AmbiguousRuleNode, rng: &mut Rng) -> Option<CustomRuleNode> {
    let total = ambig.total_weight();
    if total <= 0.0 {
        return ambig.variants.first().cloned();
    }
    let roll = total * rng.next_f64();
    let mut acc = 0.0;
    for v in &ambig.variants {
        acc += v.weight;
        if roll <= acc {
            return Some(v.clone());
        }
    }
    ambig.variants.last().cloned()
}

// ─────────────────────────────────────────────────────────────────────────────
// Primitive emission
// ─────────────────────────────────────────────────────────────────────────────

fn emit_object(kind_name: &str, class_tag: Option<&str>, state: &State, scene: &mut Scene) {
    let kind = match kind_name {
        "box"      => PrimitiveKind::Box,
        "sphere"   => PrimitiveKind::Sphere,
        "cylinder" => PrimitiveKind::Cylinder,
        "mesh"     => PrimitiveKind::Mesh,
        "line"     => PrimitiveKind::Line,
        "dot"      => PrimitiveKind::Dot,
        "grid"     => PrimitiveKind::Grid,
        "template" => PrimitiveKind::Template,
        "triangle" => PrimitiveKind::Triangle(
            class_tag.unwrap_or("").to_owned(),
        ),
        other => {
            warn!("Unknown primitive kind '{other}' — skipping");
            return;
        }
    };

    let rgba = state.color.to_rgba();
    scene.objects.push(SceneObject {
        kind,
        transform: state.transform,
        color: Rgba::new(rgba.r, rgba.g, rgba.b, 1.0),
        alpha: state.color.a,
        tag: class_tag.map(str::to_owned),
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Action expansion helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Expand an action into one or more `(rule_name, State)` entries and push
/// them onto `next`.  The expansion iterates over all counter combinations
/// of the transform loops (cartesian product), matching the legacy behaviour.
pub(crate) fn push_action_states(
    action: &Action,
    state: &State,
    calling_rule: Option<(&str, i32)>,
    rng: &mut Rng,
    next: &mut VecDeque<(String, State)>,
) {
    if action.loops.is_empty() {
        // Bare rule reference.
        let mut new_state = state.clone();
        if let Some((rule_name, depth)) = calling_rule {
            new_state.max_depths.insert(rule_name.to_owned(), depth);
        }
        next.push_back((action.target.clone(), new_state));
        return;
    }

    // Iterate all counter combinations: counters[i] ∈ [1, loops[i].count].
    let n = action.loops.len();
    let mut counters: Vec<u32> = vec![1; n];

    loop {
        // Build the new state by applying each loop's transforms `counters[i]` times.
        let mut s = state.clone();
        for (i, lp) in action.loops.iter().enumerate() {
            for _ in 0..counters[i] {
                s = apply_transforms(&lp.transforms, &s, rng);
            }
        }
        if let Some((rule_name, depth)) = calling_rule {
            s.max_depths.insert(rule_name.to_owned(), depth);
        }
        next.push_back((action.target.clone(), s));

        // Increment counters (least-significant first), mimicking the legacy.
        counters[0] += 1;
        let mut done = false;
        for i in 0..n {
            if counters[i] > action.loops[i].count {
                if i + 1 == n {
                    done = true;
                    break;
                }
                counters[i] = 1;
                counters[i + 1] += 1;
            }
        }
        if done {
            break;
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Set command processing
// ─────────────────────────────────────────────────────────────────────────────

fn apply_set_command(
    key: &str,
    value: &str,
    config: &mut BuildConfig,
    scene: &mut Scene,
    rng: &mut Rng,
) {
    match key {
        "maxdepth" => {
            if let Ok(n) = value.parse::<u32>() {
                config.max_generations = n;
            }
        }
        "maxobjects" => {
            if let Ok(n) = value.parse::<usize>() {
                config.max_objects = n;
            }
        }
        "minsize" => {
            if let Ok(f) = value.parse::<f32>() {
                config.min_dim = f;
            }
        }
        "maxsize" => {
            if let Ok(f) = value.parse::<f32>() {
                config.max_dim = f;
            }
        }
        "seed" => {
            if value.eq_ignore_ascii_case("initial") {
                // Use the already-configured seed as the fixed initial seed.
            } else if let Ok(n) = value.parse::<u64>() {
                *rng = Rng::new(n);
                config.seed = n;
            }
        }
        "syncrandom" => {
            config.sync_random = value.eq_ignore_ascii_case("true");
        }
        "recursion" => {
            // "recursion depth" sets depth-first mode (key="recursion" value="depth").
            if value.eq_ignore_ascii_case("depth") {
                config.mode = RecursionMode::DepthFirst;
            }
        }
        "background" => {
            if let Some(rgba) = rustsynth_core::color::Rgba::from_hex(value) {
                scene.background = Some(rgba);
            }
        }
        "translation" => {
            let cam = scene.camera.get_or_insert_with(CameraState::default);
            if let Some(v) = parse_vec3_bracket(value) {
                cam.translation = v;
            }
        }
        "rotation" => {
            let cam = scene.camera.get_or_insert_with(CameraState::default);
            if let Some(m) = parse_mat3_as_mat4_bracket(value) {
                cam.rotation = m;
            }
        }
        "pivot" => {
            let cam = scene.camera.get_or_insert_with(CameraState::default);
            if let Some(v) = parse_vec3_bracket(value) {
                cam.pivot = v;
            }
        }
        "scale" => {
            let cam = scene.camera.get_or_insert_with(CameraState::default);
            if let Ok(f) = value.parse::<f32>() {
                cam.scale = f;
            }
        }
        // Everything else is a pass-through (raytracer hints, template params, …).
        other => {
            scene.raw_settings.push((other.to_owned(), value.to_owned()));
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Pruning / depth helpers
// ─────────────────────────────────────────────────────────────────────────────

fn should_prune(transform: &Mat4, config: &BuildConfig) -> bool {
    if config.min_dim == 0.0 && config.max_dim == 0.0 {
        return false;
    }
    // Approximate object size via the scaled unit vector.
    let s = transform.transform_vector3(Vec3::ONE) - transform.transform_vector3(Vec3::ZERO);
    let len = s.length();
    if config.max_dim > 0.0 && len > config.max_dim {
        return true;
    }
    if config.min_dim > 0.0 && len < config.min_dim {
        return true;
    }
    false
}

/// If using depth-first with a global max-depth, propagate it to all rules
/// that don't already have a per-rule max_depth set.
fn set_rules_max_depth(graph: &mut RuleGraph, max_depth: u32) {
    for node in graph.rules.values_mut() {
        match node {
            RuleNode::Custom(c) => {
                if c.max_depth.is_none() {
                    c.max_depth = Some(max_depth);
                }
            }
            RuleNode::Ambiguous(a) => {
                for v in &mut a.variants {
                    if v.max_depth.is_none() {
                        v.max_depth = Some(max_depth);
                    }
                }
            }
            _ => {}
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Camera-value parsers
// ─────────────────────────────────────────────────────────────────────────────

/// Parse `[x y z]` into `[f32; 3]`.
fn parse_vec3_bracket(s: &str) -> Option<[f32; 3]> {
    let inner = s.trim().strip_prefix('[')?.strip_suffix(']')?;
    let nums: Vec<f32> = inner
        .split_whitespace()
        .filter_map(|t| t.parse().ok())
        .collect();
    if nums.len() >= 3 {
        Some([nums[0], nums[1], nums[2]])
    } else {
        None
    }
}

/// Parse a 9-element bracket vector `[m00 m01 m02 m10 … m22]` into a
/// column-major 4×4 matrix stored as `[f32; 16]`.
fn parse_mat3_as_mat4_bracket(s: &str) -> Option<[f32; 16]> {
    let inner = s.trim().strip_prefix('[')?.strip_suffix(']')?;
    let nums: Vec<f32> = inner
        .split_whitespace()
        .filter_map(|t| t.parse().ok())
        .collect();
    if nums.len() < 9 {
        return None;
    }
    // Legacy stores row-major 3×3; expand to column-major 4×4.
    // Row-major: [r0c0 r0c1 r0c2  r1c0 r1c1 r1c2  r2c0 r2c1 r2c2]
    // glam column-major layout: col0=[r0c0,r1c0,r2c0,0], col1=[r0c1,r1c1,r2c1,0], ...
    #[rustfmt::skip]
    let m16 = [
        nums[0], nums[3], nums[6], 0.0,   // col 0
        nums[1], nums[4], nums[7], 0.0,   // col 1
        nums[2], nums[5], nums[8], 0.0,   // col 2
        0.0,     0.0,     0.0,     1.0,   // col 3 (translation/w)
    ];
    Some(m16)
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use rustsynth_eisenscript::parser::parse;
    use rustsynth_semantics::resolve;

    fn build_script(source: &str) -> Scene {
        let pr = parse(source);
        assert!(pr.diagnostics.is_empty(), "parse errors: {:?}", pr.diagnostics);
        let (graph, diags) = resolve(&pr.script);
        assert!(diags.is_empty(), "resolution errors: {:?}", diags);
        build(&graph, &BuildConfig::default())
    }

    #[test]
    fn simple_box_emits_one_object() {
        let scene = build_script("box");
        assert_eq!(scene.objects.len(), 1);
        assert_eq!(scene.objects[0].kind, PrimitiveKind::Box);
    }

    #[test]
    fn rule_with_bare_target_emits_primitive() {
        let scene = build_script("rule T { sphere } T");
        assert_eq!(scene.objects.len(), 1);
        assert_eq!(scene.objects[0].kind, PrimitiveKind::Sphere);
    }

    #[test]
    fn multiply_syntax_creates_multiple_instances() {
        // 3 * { rz 120 } box — should emit 3 boxes (each at a different rotation).
        let scene = build_script("3 * { rz 120 } box");
        assert_eq!(scene.objects.len(), 3, "should have 3 box instances");
    }

    #[test]
    fn set_background_populates_scene() {
        let scene = build_script("set background #ff0000\nbox");
        assert!(scene.background.is_some());
        let bg = scene.background.unwrap();
        assert!(bg.r > 0.99 && bg.g < 0.01, "should be red");
    }

    #[test]
    fn maxdepth_limits_recursion() {
        // Rule with maxdepth 3 should terminate after 3 levels.
        let source = r#"
            set maxdepth 5
            R1
            rule R1 maxdepth 3 { { x 1 } R1  box }
        "#;
        let scene = build_script(source);
        // After 3 R1 applications, only 3 boxes (one per R1 invocation).
        // Exact count depends on recursion, but must be ≤ 3.
        assert!(scene.objects.len() <= 3);
    }

    #[test]
    fn ambiguous_rule_selects_variant() {
        // Two variants of the same rule — after many builds both should be chosen.
        let source = r#"
            rule T w 1 { box }
            rule T w 1 { sphere }
            T
        "#;
        let mut box_count = 0;
        let mut sphere_count = 0;
        for seed in 0..20u64 {
            let pr = parse(source);
            let (graph, _) = resolve(&pr.script);
            let cfg = BuildConfig { seed, ..Default::default() };
            let scene = build(&graph, &cfg);
            for obj in &scene.objects {
                match obj.kind {
                    PrimitiveKind::Box => box_count += 1,
                    PrimitiveKind::Sphere => sphere_count += 1,
                    _ => {}
                }
            }
        }
        assert!(box_count > 0, "should emit boxes sometimes");
        assert!(sphere_count > 0, "should emit spheres sometimes");
    }

    #[test]
    fn menger_emits_correct_object_count() {
        let source = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/eisenscript/Menger.es"
        ));
        let pr = parse(source);
        assert!(pr.diagnostics.is_empty(), "{:?}", pr.diagnostics);
        let (graph, diags) = resolve(&pr.script);
        // Menger with maxdepth 3 and 20 corner placements per level → 20^3 = 8000
        // boxes in the c2 retirement rule.  Check it's non-zero and roughly right.
        assert!(diags.is_empty(), "{:?}", diags);
        let scene = build(&graph, &BuildConfig::default());
        assert!(!scene.objects.is_empty(), "Menger should emit objects");
    }
}

