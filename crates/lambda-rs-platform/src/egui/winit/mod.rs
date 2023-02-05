//! Custom integration between [egui](https://crates.io/crates/egui)
//! and [winit](https://crates.io/crates/winit).

pub mod input;

use egui::{
  Context,
  Modifiers,
  RawInput,
};
use winit::{
  dpi::PhysicalPosition,
  event::{
    ElementState,
    Event,
    MouseButton,
    WindowEvent,
  },
};

use self::input::winit_to_egui_mouse_button;
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
      mouse_position: None,
      cursor_button_active: false,
      current_pixels_per_point: 1.0,
      emulate_touch_screen: false,
    }
  }

  fn process_mouse_input(&mut self, state: ElementState, button: MouseButton) {
    match self.mouse_position {
      Some(position) => match winit_to_egui_mouse_button(button) {
        Some(button) => {
          let is_pressed = state == winit::event::ElementState::Pressed;
        }
        None => {
          logging::warn!("Couldn't convert the winit mouse button to an egui mouse button. Ignoring input.");
        }
      },
      None => {
        logging::debug!(
          "Mouse position not within the bounds of the window. Ignoring input."
        );
      }
    }
  }

  fn process_mouse_movement(
    &mut self,
    physical_mouse_position: PhysicalPosition<f64>,
  ) {
    let normalized_position = egui::pos2(
      physical_mouse_position.x as f32 / self.current_pixels_per_point,
      physical_mouse_position.y as f32 / self.current_pixels_per_point,
    );

    self.mouse_position = Some(normalized_position);

    match self.emulate_touch_screen {
      true => if self.cursor_button_active {},
      false => {}
    }
  }

  pub fn on_event<UserEventType: 'static>(
    &mut self,
    event: &Event<UserEventType>,
  ) -> EventResult {
    return match event {
      Event::NewEvents(_) => todo!(),
      Event::WindowEvent { window_id, event } => match event {
        // File events.
        WindowEvent::DroppedFile(path) => {
          self.internal_egui_input.dropped_files.clear();
          self
            .internal_egui_input
            .dropped_files
            .push(egui::DroppedFile {
              path: Some(path.clone()),
              ..Default::default()
            });
          return EventResult {
            redraw: true,
            processed: false,
          };
        }
        WindowEvent::HoveredFile(path) => {
          self
            .internal_egui_input
            .hovered_files
            .push(egui::HoveredFile {
              path: Some(path.clone()),
              ..Default::default()
            });
          return EventResult {
            redraw: true,
            processed: false,
          };
        }
        WindowEvent::HoveredFileCancelled => {
          self.internal_egui_input.hovered_files.clear();
          return EventResult {
            redraw: true,
            processed: false,
          };
        }
        // Keyboard events.
        WindowEvent::ReceivedCharacter(_) => todo!(),
        WindowEvent::KeyboardInput {
          device_id,
          input,
          is_synthetic,
        } => todo!(),
        WindowEvent::ModifiersChanged(state) => {
          self.internal_egui_input.modifiers.alt = state.alt();
          self.internal_egui_input.modifiers.ctrl = state.ctrl();
          self.internal_egui_input.modifiers.shift = state.shift();
          self.internal_egui_input.modifiers.mac_cmd =
            cfg!(target_os = "macos") && state.logo();
          self.internal_egui_input.modifiers.command =
            match cfg!(target_os = "macos") {
              true => state.logo(),
              false => state.ctrl(),
            };

          return EventResult {
            redraw: true,
            processed: false,
          };
        }
        WindowEvent::Ime(_) => todo!(),
        WindowEvent::CursorMoved {
          device_id,
          position,
          modifiers,
        } => todo!(),

        // Mouse input events
        WindowEvent::MouseInput {
          device_id,
          state,
          button,
          modifiers,
        } => {
          self.process_mouse_input(state.clone(), button.clone());
          let processed = self.internal_egui_context.wants_pointer_input();
          return EventResult {
            processed,
            redraw: true,
          };
        }
        WindowEvent::MouseWheel {
          device_id,
          delta,
          phase,
          modifiers,
        } => todo!(),
        WindowEvent::CursorLeft { .. } => {
          self.mouse_position = None;
          self
            .internal_egui_input
            .events
            .push(egui::Event::PointerGone);
          return EventResult {
            processed: false,
            redraw: true,
          };
        }

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
          return EventResult {
            processed: false,
            redraw: true,
          };
        }
        WindowEvent::Focused(focused) => {
          self.internal_egui_input.has_focus = *focused;
          match focused {
            false => self.internal_egui_input.modifiers = Modifiers::default(),
            _ => {}
          }
          return EventResult {
            processed: false,
            redraw: true,
          };
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