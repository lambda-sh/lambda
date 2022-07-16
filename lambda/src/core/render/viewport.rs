use lambda_platform::gfx;

#[derive(Debug, Clone, PartialEq)]
pub struct Viewport {
  viewport: gfx::viewport::ViewPort,
}

impl Viewport {
  /// Convert the viewport into a gfx platform viewport.
  // TODO(vmarcella): implement this using Into<PlatformViewPort>
  pub fn into_gfx_viewport(self) -> gfx::viewport::ViewPort {
    return self.viewport;
  }
}

/// Builder for viewports that are used to render a frame within the RenderContext.
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
