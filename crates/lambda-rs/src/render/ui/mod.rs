pub use lambda_platform::egui;

pub enum GridDirection {
  Horizontal,
  Vertical,
}

pub enum UIElement {
  Button {
    text: String,
    width: f32,
    height: f32,
    on_click: Option<fn()>,
  },
  Grid {
    rows: usize,
    columns: usize,
    width: f32,
    height: f32,
    direction: GridDirection,
    children: Vec<UIElement>,
  },
}

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
