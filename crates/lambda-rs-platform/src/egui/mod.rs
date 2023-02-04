use egui::{
  Context,
  RawInput,
};

pub mod winit;

pub struct EguiContext {
  internal_egui_input: RawInput,
  internal_egui_context: Context,
}
