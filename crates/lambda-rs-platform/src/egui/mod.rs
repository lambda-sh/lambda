use egui::{
  Context,
  Pos2,
  RawInput,
  TouchDeviceId,
};

pub mod gfx;
pub mod winit;

/// A context for managing egui input & rendering.
pub struct EguiContext {
  input_handler: RawInput,
  context: Context,
  mouse_position: Option<Pos2>,
  mouse_button_active: bool,
  current_pixels_per_point: f32,
  emulate_touch_screen: bool,
  active_touch_device: Option<TouchDeviceId>,
}
