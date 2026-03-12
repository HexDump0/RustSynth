//! GPU-side types shared between Rust and WGSL.
//!
//! All types derive `bytemuck::Pod + Zeroable` for direct upload to GPU buffers.

use bytemuck::{Pod, Zeroable};

/// Per-vertex data sent to the vertex shader.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    /// World-space position.
    pub position: [f32; 3],
    /// World-space normal.
    pub normal: [f32; 3],
    /// RGBA color (premultiplied alpha not required).
    pub color: [f32; 4],
}

impl Vertex {
    pub const ATTRIBS: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
        0 => Float32x3,  // position
        1 => Float32x3,  // normal
        2 => Float32x4,  // color
    ];

    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// Camera uniform block — uploaded once per frame to binding 0.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct CameraUniform {
    /// Combined view-projection matrix (column-major).
    pub view_proj: [f32; 16],
    /// Camera eye position in world space (padded to 16 bytes).
    pub eye_pos: [f32; 4],
}
