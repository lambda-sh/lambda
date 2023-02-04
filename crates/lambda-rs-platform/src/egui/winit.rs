//! Custom integration between [egui](https://crates.io/crates/egui)
//! and [winit](https://crates.io/crates/winit).

use egui::{
  Context,
  RawInput,
};
use winit::event::{
  ElementState,
  MouseButton,
};

impl super::EguiContext {
  /// Create a new input manager prepped for winit usage.
  pub fn new() -> Self {
    Self {
      internal_egui_input: RawInput {
        has_focus: false,
        ..Default::default()
      },
      internal_egui_context: Context::default(),
    }
  }

  pub fn on_event<UserEventType: 'static>(
    &self,
    event: &winit::event::Event<UserEventType>,
  ) {
    match event {
      winit::event::Event::NewEvents(_) => todo!(),
      winit::event::Event::WindowEvent { window_id, event } => match event {
        winit::event::WindowEvent::Resized(_) => todo!(),
        winit::event::WindowEvent::Moved(_) => todo!(),
        winit::event::WindowEvent::CloseRequested => todo!(),
        winit::event::WindowEvent::Destroyed => todo!(),
        winit::event::WindowEvent::DroppedFile(_) => todo!(),
        winit::event::WindowEvent::HoveredFile(_) => todo!(),
        winit::event::WindowEvent::HoveredFileCancelled => todo!(),
        winit::event::WindowEvent::ReceivedCharacter(_) => todo!(),
        winit::event::WindowEvent::Focused(_) => todo!(),
        winit::event::WindowEvent::KeyboardInput {
          device_id,
          input,
          is_synthetic,
        } => todo!(),
        winit::event::WindowEvent::ModifiersChanged(_) => todo!(),
        winit::event::WindowEvent::Ime(_) => todo!(),
        winit::event::WindowEvent::CursorMoved {
          device_id,
          position,
          modifiers,
        } => todo!(),
        winit::event::WindowEvent::CursorEntered { device_id } => todo!(),
        winit::event::WindowEvent::CursorLeft { device_id } => todo!(),
        winit::event::WindowEvent::MouseWheel {
          device_id,
          delta,
          phase,
          modifiers,
        } => todo!(),
        winit::event::WindowEvent::MouseInput {
          device_id,
          state,
          button,
          modifiers,
        } => todo!(),
        winit::event::WindowEvent::TouchpadPressure {
          device_id,
          pressure,
          stage,
        } => todo!(),
        winit::event::WindowEvent::AxisMotion {
          device_id,
          axis,
          value,
        } => todo!(),
        winit::event::WindowEvent::Touch(_) => todo!(),
        winit::event::WindowEvent::ScaleFactorChanged {
          scale_factor,
          new_inner_size,
        } => todo!(),
        winit::event::WindowEvent::ThemeChanged(_) => todo!(),
        winit::event::WindowEvent::Occluded(_) => todo!(),
      },
      winit::event::Event::DeviceEvent { device_id, event } => todo!(),
      winit::event::Event::UserEvent(_) => todo!(),
      winit::event::Event::Suspended => todo!(),
      winit::event::Event::Resumed => todo!(),
      winit::event::Event::MainEventsCleared => todo!(),
      winit::event::Event::RedrawRequested(_) => todo!(),
      winit::event::Event::RedrawEventsCleared => todo!(),
      winit::event::Event::LoopDestroyed => todo!(),
    }
  }
}
