pub use lambda_platform::egui;
use lambda_platform::egui::gfx::UIElement;

/// The context for managing UI state & rendering.
pub struct UIContext {
  pub elements: Vec<UIElement>,
  egui_context: egui::EguiContext,
}

impl UIContext {
  pub fn new() -> Self {
    Self {
      elements: Vec::new(),
      egui_context: egui::EguiContext::new(),
    }
  }

  pub fn add_element(&mut self, element: UIElement) {
    self.elements.push(element);
  }

  /// Passes lower level winit events to egui for processing.
  // TODO(vmarcella): Can we change this function signature?
  pub(crate) fn on_winit_event(
    &mut self,
    event: &lambda_platform::winit::winit_exports::Event<()>,
  ) -> egui::winit::EventResult {
    self.egui_context.on_event(event)
  }
}
