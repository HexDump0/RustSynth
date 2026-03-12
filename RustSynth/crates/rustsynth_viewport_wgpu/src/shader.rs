//! Embedded WGSL shader source for the viewport pipeline.

/// The main shader module for rendering lit, coloured geometry.
pub const MAIN_SHADER: &str = r#"
// ────────────────────────────────────────────────────────────────────────────
// RustSynth viewport shader — Blinn-Phong with per-vertex color
// ────────────────────────────────────────────────────────────────────────────

struct CameraUniform {
    view_proj: mat4x4<f32>,
    eye_pos: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) color: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(in.position, 1.0);
    out.world_pos = in.position;
    out.world_normal = in.normal;
    out.color = in.color;
    return out;
}

// ── Fragment shader — Blinn-Phong ──────────────────────────────────────────

const AMBIENT_STRENGTH: f32 = 0.15;
const DIFFUSE_STRENGTH: f32 = 0.7;
const SPECULAR_STRENGTH: f32 = 0.4;
const SHININESS: f32 = 32.0;

// Two directional lights for better coverage (similar to legacy Structure Synth)
const LIGHT_DIR_0: vec3<f32> = vec3<f32>(0.3, 1.0, 0.5);
const LIGHT_DIR_1: vec3<f32> = vec3<f32>(-0.5, 0.3, -0.8);
const LIGHT_COLOR: vec3<f32> = vec3<f32>(1.0, 1.0, 1.0);

fn blinn_phong(normal: vec3<f32>, view_dir: vec3<f32>, light_dir: vec3<f32>, base_color: vec3<f32>) -> vec3<f32> {
    let n = normalize(normal);
    let l = normalize(light_dir);
    let v = normalize(view_dir);

    // Ambient
    let ambient = AMBIENT_STRENGTH * base_color;

    // Diffuse — use abs(dot) for two-sided lighting
    let diff = max(dot(n, l), 0.0);
    let diffuse = DIFFUSE_STRENGTH * diff * base_color;

    // Specular (Blinn-Phong half-vector)
    let half_dir = normalize(l + v);
    let spec = pow(max(dot(n, half_dir), 0.0), SHININESS);
    let specular = SPECULAR_STRENGTH * spec * LIGHT_COLOR;

    return ambient + diffuse + specular;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let view_dir = normalize(camera.eye_pos.xyz - in.world_pos);
    let normal = normalize(in.world_normal);
    let base = in.color.rgb;

    // Accumulate two lights
    var lit = blinn_phong(normal, view_dir, LIGHT_DIR_0, base);
    lit = lit + blinn_phong(normal, view_dir, LIGHT_DIR_1, base) * 0.4;

    // Clamp to avoid over-bright
    lit = clamp(lit, vec3<f32>(0.0), vec3<f32>(1.0));

    return vec4<f32>(lit, in.color.a);
}
"#;
