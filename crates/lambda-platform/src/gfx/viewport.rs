#[derive(Debug, Clone)]
pub struct ViewPort {
  viewport: gfx_hal::pso::Viewport,
}

impl ViewPort {}

pub struct ViewPortBuilder {
  x: i16,
  y: i16,
}

impl ViewPortBuilder {
  pub fn new() -> Self {
    return ViewPortBuilder { x: 0, y: 0 };
  }

  pub fn with_coordinates(mut self, x: i16, y: i16) -> Self {
    self.x = x;
    self.y = y;
    return self;
  }

  /// Build a viewport to use for viewing an entire surface.
  pub fn build<RenderBackend: gfx_hal::Backend>(
    self,
    surface: &super::surface::Surface<RenderBackend>,
  ) -> ViewPort {
    let [width, height] = surface
      .size()
      .expect("Surface doesn't have a size for the viewport to use!");

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

pub mod internal {
  /// Return the gfx_hal viewport.
  pub fn viewport_for(viewport: &super::ViewPort) -> gfx_hal::pso::Viewport {
    return viewport.viewport.clone();
  }
}
