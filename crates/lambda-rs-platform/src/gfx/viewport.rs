//! viewport.rs - Low level viewport management for the render context.

/// Viewport is a rectangle that defines the area of the screen that will be
/// rendered to. For instance, if the viewport is set to x=0, y=0, width=100,
/// height=100, then only the top left 100x100 pixels of the screen will be
/// rendered to.
#[derive(Debug, Clone, PartialEq)]
pub struct ViewPort {
  viewport: gfx_hal::pso::Viewport,
}

impl ViewPort {
  /// Get the internal viewport object.
  pub(super) fn internal_viewport(&self) -> gfx_hal::pso::Viewport {
    return self.viewport.clone();
  }
}

/// A builder for `Viewport`.
pub struct ViewPortBuilder {
  x: i16,
  y: i16,
}

impl ViewPortBuilder {
  /// Create a new `ViewportBuilder`.
  pub fn new() -> Self {
    return ViewPortBuilder { x: 0, y: 0 };
  }

  /// Specify a viewport with specific coordinates
  pub fn with_coordinates(mut self, x: i16, y: i16) -> Self {
    self.x = x;
    self.y = y;
    return self;
  }

  /// Build a viewport to use for viewing an entire surface.
  pub fn build(self, width: u32, height: u32) -> ViewPort {
    // The viewport currently renders to the entire size of the surface. and has
    // a non-configurable depth
    return ViewPort {
      viewport: gfx_hal::pso::Viewport {
        rect: gfx_hal::pso::Rect {
          x: self.x,
          y: self.y,
          w: width as i16,
          h: height as i16,
        },
        depth: 0.0..1.0,
      },
    };
  }
}

#[cfg(test)]
pub mod tests {
  /// Test the viewport builder in it's default state.
  #[test]
  fn viewport_builder_default_initial_state() {
    let viewport_builder = super::ViewPortBuilder::new();
    assert_eq!(viewport_builder.x, 0);
    assert_eq!(viewport_builder.y, 0);
  }

  /// Test that the viewport builder can be configured with specific
  /// coordinates.
  #[test]
  fn viewport_builder_with_coordinates() {
    let viewport_builder =
      super::ViewPortBuilder::new().with_coordinates(10, 10);
    assert_eq!(viewport_builder.x, 10);
    assert_eq!(viewport_builder.y, 10);
  }

  #[test]
  fn viewport_builder_builds_successfully() {
    let viewport_builder =
      super::ViewPortBuilder::new().with_coordinates(10, 10);
    let viewport = viewport_builder.build(100, 100);
    assert_eq!(viewport.viewport.rect.x, 10);
    assert_eq!(viewport.viewport.rect.y, 10);
    assert_eq!(viewport.viewport.rect.w, 100);
    assert_eq!(viewport.viewport.rect.h, 100);
  }
}
