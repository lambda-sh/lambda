use lambda_platform::gfx;
pub struct Viewport {
  viewport: gfx::viewport::ViewPort,
}

pub struct ViewportBuilder {
  x: i16,
  y: i16,
}

impl ViewportBuilder {
  pub fn new() -> Self {
    return Self { x: 0, y: 0 };
  }

  /// Builds a viewport that can be used for defining
  pub fn build(self, width: u32, height: u32) -> Viewport {
    let viewport = gfx::viewport::ViewPortBuilder::new()
      .with_coordinates(self.x, self.y)
      .build(width, height);

    return Viewport { viewport };
  }
}
