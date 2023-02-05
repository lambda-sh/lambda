use egui::{
  Context,
  Pos2,
  RawInput,
};

pub mod winit;

/// A context for managing egui input & rendering.
pub struct EguiContext {
  internal_egui_input: RawInput,
  internal_egui_context: Context,
  internal_cursor_position: Option<Pos2>,
}
