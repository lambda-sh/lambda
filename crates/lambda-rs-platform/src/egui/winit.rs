//! Custom integration between [egui](https://crates.io/crates/egui)
//! and [winit](https://crates.io/crates/winit).

use egui::{
  Context,
  RawInput,
};
use winit::event::{
  ElementState,
  Event,
  MouseButton,
  WindowEvent,
};
pub struct EventResult {
  pub consumed: bool,
  pub redraw: bool,
}

impl super::EguiContext {
  /// Create a new input manager prepped for winit usage.
  pub fn new() -> Self {
    Self {
      internal_egui_input: RawInput {
        has_focus: false,
        ..Default::default()
      },
      internal_egui_context: Context::default(),
      internal_cursor_position: None,
    }
  }

  fn process_mouse_input(&mut self, state: ElementState, button: MouseButton) {}

  pub fn on_event<UserEventType: 'static>(
    &mut self,
    event: &Event<UserEventType>,
  ) -> EventResult {
    return match event {
      Event::NewEvents(_) => todo!(),
      Event::WindowEvent { window_id, event } => match event {
        WindowEvent::Resized(_) => todo!(),
        WindowEvent::Moved(_) => todo!(),
        WindowEvent::CloseRequested => todo!(),
        WindowEvent::Destroyed => todo!(),
        WindowEvent::DroppedFile(_) => todo!(),
        WindowEvent::HoveredFile(_) => todo!(),
        WindowEvent::HoveredFileCancelled => todo!(),
        WindowEvent::ReceivedCharacter(_) => todo!(),
        WindowEvent::Focused(_) => todo!(),
        WindowEvent::KeyboardInput {
          device_id,
          input,
          is_synthetic,
        } => todo!(),
        WindowEvent::ModifiersChanged(_) => todo!(),
        WindowEvent::Ime(_) => todo!(),
        WindowEvent::CursorMoved {
          device_id,
          position,
          modifiers,
        } => todo!(),
        WindowEvent::CursorEntered { device_id } => todo!(),
        WindowEvent::CursorLeft { device_id } => todo!(),
        WindowEvent::MouseWheel {
          device_id,
          delta,
          phase,
          modifiers,
        } => todo!(),
        WindowEvent::MouseInput {
          device_id,
          state,
          button,
          modifiers,
        } => {
          self.process_mouse_input(state.clone(), button.clone());
          EventResult {
            consumed: self.internal_egui_context.wants_pointer_input(),
            redraw: true,
          }
        }
        WindowEvent::TouchpadPressure {
          device_id,
          pressure,
          stage,
        } => todo!(),
        WindowEvent::AxisMotion {
          device_id,
          axis,
          value,
        } => todo!(),
        WindowEvent::Touch(_) => todo!(),
        WindowEvent::ScaleFactorChanged {
          scale_factor,
          new_inner_size,
        } => {
          let pixels_per_point = *scale_factor as f32;
          self.internal_egui_input.pixels_per_point = Some(pixels_per_point);
          self
            .internal_egui_context
            .set_pixels_per_point(pixels_per_point);
          EventResult {
            consumed: false,
            redraw: true,
          }
        }
        WindowEvent::ThemeChanged(_) => todo!(),
        WindowEvent::Occluded(_) => todo!(),
      },
      Event::DeviceEvent { device_id, event } => todo!(),
      Event::UserEvent(_) => todo!(),
      Event::Suspended => todo!(),
      Event::Resumed => todo!(),
      Event::MainEventsCleared => todo!(),
      Event::RedrawRequested(_) => todo!(),
      Event::RedrawEventsCleared => todo!(),
      Event::LoopDestroyed => todo!(),
    };
  }
}
