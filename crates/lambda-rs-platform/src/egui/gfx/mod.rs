//! Support module for rendering `egui` elements within our rendering
//! infrastructure.
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

impl UIElement {
  /// Renders the UI element
  pub(crate) fn render(&mut self, ui_for_frame: &mut egui::Ui) {
    match self {
      UIElement::Button {
        text,
        width,
        height,
        on_click,
      } => {
        if ui_for_frame.button(text.as_str()).clicked() {
          match on_click {
            Some(on_click) => on_click(),
            None => {}
          }
        }
      }
      UIElement::Grid {
        rows,
        columns,
        width,
        height,
        direction,
        children,
      } => todo!(),
    }
  }
}
