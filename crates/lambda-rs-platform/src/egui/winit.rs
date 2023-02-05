//! Custom integration between [egui](https://crates.io/crates/egui)
//! and [winit](https://crates.io/crates/winit).

use egui::{
  Context,
  Modifiers,
  RawInput,
};
use winit::event::{
  ElementState,
  Event,
  MouseButton,
  WindowEvent,
};
pub struct EventResult {
  pub processed: bool,
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
      cursor_position: None,
      cursor_button_active: false,
      current_pixels_per_point: 1.0,
      emulate_touch_screen: false,
    }
  }

  fn process_mouse_input(&mut self, state: ElementState, button: MouseButton) {
    if let Some(position) = self.cursor_position {}
  }

  pub fn on_event<UserEventType: 'static>(
    &mut self,
    event: &Event<UserEventType>,
  ) -> EventResult {
    return match event {
      Event::NewEvents(_) => todo!(),
      Event::WindowEvent { window_id, event } => match event {
        // File events.
        WindowEvent::DroppedFile(_) => todo!(),
        WindowEvent::HoveredFile(_) => todo!(),
        WindowEvent::HoveredFileCancelled => todo!(),
        // Keyboard events.
        WindowEvent::ReceivedCharacter(_) => todo!(),
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
        WindowEvent::CursorLeft { device_id } => todo!(),
        // Mouse input events
        WindowEvent::MouseInput {
          device_id,
          state,
          button,
          modifiers,
        } => {
          self.process_mouse_input(state.clone(), button.clone());
          EventResult {
            processed: self.internal_egui_context.wants_pointer_input(),
            redraw: true,
          }
        }
        WindowEvent::MouseWheel {
          device_id,
          delta,
          phase,
          modifiers,
        } => todo!(),

        // Repaint events
        WindowEvent::CloseRequested
        | WindowEvent::CursorEntered { .. }
        | WindowEvent::Destroyed
        | WindowEvent::ThemeChanged(_)
        | WindowEvent::Occluded(_)
        | WindowEvent::Resized(_)
        | WindowEvent::TouchpadPressure { .. } => EventResult {
          processed: false,
          redraw: true,
        },

        // Noop events
        WindowEvent::Moved(_) | WindowEvent::AxisMotion { .. } => EventResult {
          processed: false,
          redraw: false,
        },
        WindowEvent::Touch(_) => todo!(),

        // Window Events
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
            processed: false,
            redraw: true,
          }
        }
        WindowEvent::Focused(focused) => {
          self.internal_egui_input.has_focus = *focused;
          match focused {
            false => self.internal_egui_input.modifiers = Modifiers::default(),
            _ => {}
          }
          EventResult {
            processed: false,
            redraw: true,
          }
        }
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
