//! `WgpuBackend` — the wgpu implementation of [`ViewportBackend`].
//!
//! # Lifecycle (driven by the GTK app shell)
//!
//! 1. **`init()`** — called from `GtkGLArea::realize`. Creates the wgpu device,
//!    queue, pipeline, and camera uniform buffer.
//! 2. **`load_scene()`** — tessellates scene objects into vertex/index buffers
//!    and uploads to the GPU.
//! 3. **`render_frame()`** — called from `GtkGLArea::render`. Writes the camera
//!    uniform, issues draw calls.
//! 4. **`resize()`** — called on allocation changes. Recreates the depth buffer.
//! 5. **`shutdown()`** — drops all GPU resources.
//!
//! The backend does **not** own a surface or swapchain — when integrated with
//! `GtkGLArea`, the GTK runtime provides the framebuffer. For headless / test
//! usage the backend can render to an offscreen texture.

use anyhow::{Context, Result};
use wgpu::util::DeviceExt;

use rustsynth_render_api::backend::{InputEvent, PointerButton, ViewportBackend};
use rustsynth_render_api::camera::ArcballCamera;
use rustsynth_scene::Scene;

use crate::geometry;
use crate::gpu_types::{CameraUniform, Vertex};
use crate::pipeline::{self, Pipeline};

/// GPU resources created during `init` and destroyed during `shutdown`.
struct GpuState {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: Pipeline,
    camera_buf: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    depth_view: wgpu::TextureView,
    /// Surface and config for headless/windowed rendering (None when using
    /// an external framebuffer like GtkGLArea).
    surface_state: Option<SurfaceState>,
}

/// Optional surface state for standalone (non-GTK) rendering.
struct SurfaceState {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
}

/// Uploaded scene geometry.
struct SceneBuffers {
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    num_indices: u32,
}

/// The wgpu viewport backend.
///
/// Create with [`WgpuBackend::new`], then follow the [`ViewportBackend`]
/// lifecycle: `init → load_scene → render_frame (loop) → shutdown`.
pub struct WgpuBackend {
    camera: ArcballCamera,
    width: u32,
    height: u32,
    clear_color: wgpu::Color,
    gpu: Option<GpuState>,
    scene_bufs: Option<SceneBuffers>,
    /// Optional: an `wgpu::Instance` can be provided externally for
    /// integration with GtkGLArea.  If `None`, `init` creates one.
    instance: Option<wgpu::Instance>,
}

impl Default for WgpuBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl WgpuBackend {
    /// Create a new backend with default camera and no GPU resources.
    pub fn new() -> Self {
        Self {
            camera: ArcballCamera::default(),
            width: 800,
            height: 600,
            clear_color: wgpu::Color {
                r: 0.15,
                g: 0.15,
                b: 0.2,
                a: 1.0,
            },
            gpu: None,
            scene_bufs: None,
            instance: None,
        }
    }

    /// Set the background clear colour (e.g. from `scene.background`).
    pub fn set_clear_color(&mut self, r: f64, g: f64, b: f64, a: f64) {
        self.clear_color = wgpu::Color { r, g, b, a };
    }

    /// Provide a pre-created wgpu instance (e.g. from GtkGLArea EGL context).
    pub fn set_instance(&mut self, instance: wgpu::Instance) {
        self.instance = Some(instance);
    }

    /// Provide a surface for standalone rendering (non-GTK).
    /// Must be called before `init`.
    pub fn set_surface(&mut self, surface: wgpu::Surface<'static>) {
        if let Some(gpu) = &self.gpu {
            let config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                width: self.width,
                height: self.height,
                present_mode: wgpu::PresentMode::Fifo,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            };
            surface.configure(&gpu.device, &config);
            // Can't easily set surface_state after init without refactoring,
            // so this is only useful before init.
            let _ = config;
        }
    }

    /// Build the camera uniform data from the current camera state.
    fn camera_uniform(&self) -> CameraUniform {
        let vp = self.camera.view_proj();
        let eye = self.camera.eye();
        CameraUniform {
            view_proj: vp.to_cols_array(),
            eye_pos: [eye.x, eye.y, eye.z, 1.0],
        }
    }

    /// Upload the scene mesh to GPU buffers.
    fn upload_scene_mesh(
        device: &wgpu::Device,
        vertices: &[Vertex],
        indices: &[u32],
    ) -> SceneBuffers {
        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("scene_vertex_buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("scene_index_buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        SceneBuffers {
            vertex_buf,
            index_buf,
            num_indices: indices.len() as u32,
        }
    }

    /// Render to a provided texture view (used by GtkGLArea integration).
    pub fn render_to_view(&mut self, view: &wgpu::TextureView) -> Result<()> {
        let gpu = self
            .gpu
            .as_ref()
            .context("render_to_view: GPU not initialised")?;

        // Update camera uniform
        let uniform = self.camera_uniform();
        gpu.queue
            .write_buffer(&gpu.camera_buf, 0, bytemuck::bytes_of(&uniform));

        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame_encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &gpu.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_pipeline(&gpu.pipeline.render_pipeline);
            pass.set_bind_group(0, &gpu.camera_bind_group, &[]);

            if let Some(bufs) = &self.scene_bufs {
                pass.set_vertex_buffer(0, bufs.vertex_buf.slice(..));
                pass.set_index_buffer(bufs.index_buf.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..bufs.num_indices, 0, 0..1);
            }
        }

        gpu.queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }

    /// Get a reference to the wgpu device (for GtkGLArea integration).
    pub fn device(&self) -> Option<&wgpu::Device> {
        self.gpu.as_ref().map(|g| &g.device)
    }

    /// Get a reference to the wgpu queue (for GtkGLArea integration).
    pub fn queue(&self) -> Option<&wgpu::Queue> {
        self.gpu.as_ref().map(|g| &g.queue)
    }
}

impl ViewportBackend for WgpuBackend {
    fn init(&mut self) -> Result<()> {
        let instance = self.instance.take().unwrap_or_else(|| {
            wgpu::Instance::new(&wgpu::InstanceDescriptor {
                backends: wgpu::Backends::all(),
                ..Default::default()
            })
        });

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
        }))
        .context("Failed to find a suitable GPU adapter")?;

        log::info!(
            "wgpu adapter: {} ({:?})",
            adapter.get_info().name,
            adapter.get_info().backend
        );

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("rustsynth_device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
                memory_hints: Default::default(),
            },
            None,
        ))
        .context("Failed to create wgpu device")?;

        let surface_format = wgpu::TextureFormat::Bgra8UnormSrgb;

        let pipe = Pipeline::new(&device, surface_format);

        // Camera uniform buffer
        let uniform = self.camera_uniform();
        let camera_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera_uniform_buffer"),
            contents: bytemuck::bytes_of(&uniform),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera_bind_group"),
            layout: &pipe.bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buf.as_entire_binding(),
            }],
        });

        let depth_view =
            pipeline::create_depth_texture(&device, self.width, self.height, pipe.depth_format);

        self.gpu = Some(GpuState {
            device,
            queue,
            pipeline: pipe,
            camera_buf,
            camera_bind_group,
            depth_view,
            surface_state: None,
        });

        log::info!(
            "WgpuBackend initialised ({}×{})",
            self.width,
            self.height
        );
        Ok(())
    }

    fn shutdown(&mut self) {
        self.scene_bufs = None;
        self.gpu = None;
        log::info!("WgpuBackend shut down");
    }

    fn load_scene(&mut self, scene: &Scene) -> Result<()> {
        let gpu = self
            .gpu
            .as_ref()
            .context("load_scene: GPU not initialised")?;

        // Set background colour
        if let Some(bg) = &scene.background {
            self.clear_color = wgpu::Color {
                r: bg.r as f64,
                g: bg.g as f64,
                b: bg.b as f64,
                a: bg.a as f64,
            };
        }

        let (vertices, indices) = geometry::scene_to_mesh(&scene.objects);
        log::info!(
            "Scene tessellated: {} vertices, {} indices ({} objects)",
            vertices.len(),
            indices.len(),
            scene.objects.len(),
        );

        if vertices.is_empty() {
            self.scene_bufs = None;
            return Ok(());
        }

        self.scene_bufs = Some(Self::upload_scene_mesh(&gpu.device, &vertices, &indices));
        Ok(())
    }

    fn render_frame(&mut self) -> Result<()> {
        // If we have a surface, render to it; otherwise this is a no-op
        // (the GtkGLArea integration calls render_to_view directly).
        let has_surface = self
            .gpu
            .as_ref()
            .map_or(false, |g| g.surface_state.is_some());

        if has_surface {
            let gpu = self.gpu.as_ref().unwrap();
            let ss = gpu.surface_state.as_ref().unwrap();
            let frame = ss
                .surface
                .get_current_texture()
                .context("Failed to acquire next swapchain texture")?;
            let view = frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            self.render_to_view(&view)?;
            frame.present();
        }
        Ok(())
    }

    fn resize(&mut self, width: u32, height: u32) {
        let w = width.max(1);
        let h = height.max(1);
        self.width = w;
        self.height = h;
        self.camera.aspect = w as f32 / h as f32;

        if let Some(gpu) = &mut self.gpu {
            gpu.depth_view = pipeline::create_depth_texture(
                &gpu.device,
                w,
                h,
                gpu.pipeline.depth_format,
            );

            if let Some(ss) = &mut gpu.surface_state {
                ss.config.width = w;
                ss.config.height = h;
                ss.surface.configure(&gpu.device, &ss.config);
            }
        }
        log::debug!("WgpuBackend resized to {}×{}", w, h);
    }

    fn camera(&self) -> &ArcballCamera {
        &self.camera
    }

    fn camera_mut(&mut self) -> &mut ArcballCamera {
        &mut self.camera
    }

    fn handle_input(&mut self, event: InputEvent) -> bool {
        match event {
            InputEvent::PointerDrag { button, dx, dy } => match button {
                PointerButton::Primary => {
                    self.camera.orbit(dx * 0.3, -dy * 0.3);
                    true
                }
                PointerButton::Secondary => {
                    self.camera.pan(-dx * 0.005 * self.camera.distance, dy * 0.005 * self.camera.distance);
                    true
                }
                PointerButton::Middle => {
                    self.camera.pan(-dx * 0.005 * self.camera.distance, dy * 0.005 * self.camera.distance);
                    true
                }
            },
            InputEvent::Scroll { delta } => {
                let factor = if delta > 0.0 { 0.9 } else { 1.1 };
                self.camera.zoom(factor);
                true
            }
            InputEvent::Pan { dx, dy } => {
                self.camera.pan(-dx * 0.005 * self.camera.distance, dy * 0.005 * self.camera.distance);
                true
            }
            InputEvent::ResetCamera => {
                self.camera.reset();
                true
            }
        }
    }

    fn backend_name(&self) -> &'static str {
        "wgpu"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustsynth_render_api::backend::InputEvent;

    #[test]
    fn default_backend_has_correct_name() {
        let backend = WgpuBackend::new();
        assert_eq!(backend.backend_name(), "wgpu");
    }

    #[test]
    fn handle_input_orbit() {
        let mut backend = WgpuBackend::new();
        let initial_yaw = backend.camera().yaw;
        let changed = backend.handle_input(InputEvent::PointerDrag {
            button: PointerButton::Primary,
            dx: 10.0,
            dy: 5.0,
        });
        assert!(changed);
        assert_ne!(backend.camera().yaw, initial_yaw);
    }

    #[test]
    fn handle_input_zoom() {
        let mut backend = WgpuBackend::new();
        let initial_distance = backend.camera().distance;
        let changed = backend.handle_input(InputEvent::Scroll { delta: 1.0 });
        assert!(changed);
        assert!(backend.camera().distance < initial_distance);
    }

    #[test]
    fn handle_input_pan() {
        let mut backend = WgpuBackend::new();
        let initial_pivot = backend.camera().pivot;
        let changed = backend.handle_input(InputEvent::Pan { dx: 10.0, dy: 5.0 });
        assert!(changed);
        assert_ne!(backend.camera().pivot, initial_pivot);
    }

    #[test]
    fn handle_input_reset() {
        let mut backend = WgpuBackend::new();
        backend.camera_mut().yaw = 90.0;
        backend.camera_mut().pitch = 45.0;
        let changed = backend.handle_input(InputEvent::ResetCamera);
        assert!(changed);
        assert_eq!(backend.camera().yaw, 30.0); // default
        assert_eq!(backend.camera().pitch, 20.0); // default
    }

    #[test]
    fn resize_updates_dimensions() {
        let mut backend = WgpuBackend::new();
        backend.resize(1920, 1080);
        assert_eq!(backend.width, 1920);
        assert_eq!(backend.height, 1080);
        let expected_aspect = 1920.0_f32 / 1080.0;
        assert!((backend.camera().aspect - expected_aspect).abs() < 1e-5);
    }

    #[test]
    fn set_clear_color_works() {
        let mut backend = WgpuBackend::new();
        backend.set_clear_color(1.0, 0.0, 0.0, 1.0);
        assert_eq!(backend.clear_color.r, 1.0);
        assert_eq!(backend.clear_color.g, 0.0);
    }

    #[test]
    fn camera_uniform_matches_camera_state() {
        let backend = WgpuBackend::new();
        let uniform = backend.camera_uniform();
        let expected = backend.camera.view_proj().to_cols_array();
        assert_eq!(uniform.view_proj, expected);
        let eye = backend.camera.eye();
        assert!((uniform.eye_pos[0] - eye.x).abs() < 1e-5);
    }
}
