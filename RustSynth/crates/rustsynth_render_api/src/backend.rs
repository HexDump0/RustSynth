//! `ViewportBackend` — the trait every viewport backend must implement.
//!
//! # Lifecycle
//!
//! ```text
//! init()  ──▶  load_scene()  ──▶  render_frame()  (loop)
//!                                     │
//!                    handle_input() ──┘   resize() at any time
//!                                     │
//!                               shutdown()
//! ```
//!
//! The app shell is responsible for calling `handle_input` with pointer/keyboard
//! events and `resize` whenever the GTK drawing area changes dimensions.
//! The backend drives its own render resources; the app shell drives the loop.

use rustsynth_scene::Scene;
use crate::camera::ArcballCamera;

/// A pointer / mouse input event forwarded from the GTK layer.
#[derive(Debug, Clone)]
pub enum PointerButton {
    Primary,
    Secondary,
    Middle,
}

/// Raw input events that the app shell forwards to the viewport backend.
#[derive(Debug, Clone)]
pub enum InputEvent {
    /// Mouse/pointer drag (delta in logical pixels).
    PointerDrag {
        button: PointerButton,
        dx: f32,
        dy: f32,
    },
    /// Mouse/pointer scroll (positive = zoom in, negative = zoom out).
    Scroll { delta: f32 },
    /// Middle-button or shift-drag pan.
    Pan { dx: f32, dy: f32 },
    /// Request a full camera reset.
    ResetCamera,
}

/// Trait for a realtime viewport backend.
///
/// The app shell creates a backend, feeds it scenes, and calls `render_frame`
/// inside the GtkGLArea `render` signal. The backend owns its own rendering
/// resources (GPU buffers, shaders, etc.) and is the only object allowed to
/// touch them.
///
/// All methods return `anyhow::Result` so the app shell can surface errors
/// without depending on a specific backend error type.
pub trait ViewportBackend {
    // ── Lifecycle ────────────────────────────────────────────────────────────

    /// Initialize backend, creating all GPU resources.
    ///
    /// Called once from the GtkGLArea `realize` signal.
    fn init(&mut self) -> anyhow::Result<()>;

    /// Destroy all GPU resources.
    ///
    /// Called from the GtkGLArea `unrealize` signal. After this call the
    /// backend must be ready to `init` again.
    fn shutdown(&mut self);

    // ── Scene management ─────────────────────────────────────────────────────

    /// Replace the displayed scene with `scene`.
    ///
    /// The backend uploads all geometry and initialises any per-object state.
    /// The previous scene is discarded. Implementations should avoid
    /// allocating per-frame; do all allocation here.
    fn load_scene(&mut self, scene: &Scene) -> anyhow::Result<()>;

    // ── Render loop ───────────────────────────────────────────────────────────

    /// Render a single frame into the current framebuffer.
    ///
    /// Called from the GtkGLArea `render` signal. The surface is already
    /// current when this is invoked.
    fn render_frame(&mut self) -> anyhow::Result<()>;

    /// Called when the drawing area is resized (in physical pixels).
    fn resize(&mut self, width: u32, height: u32);

    // ── Camera ────────────────────────────────────────────────────────────────

    /// Return an immutable reference to the active camera.
    fn camera(&self) -> &ArcballCamera;

    /// Return a mutable reference to the active camera.
    fn camera_mut(&mut self) -> &mut ArcballCamera;

    /// Process an input event, updating the camera or other state.
    ///
    /// Returns `true` if the event caused a visual change and a new frame
    /// should be queued (i.e. GtkGLArea::queue_render).
    fn handle_input(&mut self, event: InputEvent) -> bool;

    // ── Introspection ─────────────────────────────────────────────────────────

    /// Human-readable backend identifier (e.g. `"wgpu-gl"`, `"opengl"`).
    fn backend_name(&self) -> &'static str;
}
