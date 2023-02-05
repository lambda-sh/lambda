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
  mouse_position: Option<Pos2>,
  cursor_button_active: bool,
  current_pixels_per_point: f32,
  emulate_touch_screen: bool,
}
