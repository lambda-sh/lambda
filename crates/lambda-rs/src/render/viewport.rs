//! Viewport and scissor state for a render pass.
//!
//! A `Viewport` applies both viewport and scissor rectangles to the active
//! render pass. Coordinates are specified in pixels with origin at the
//! top‑left of the surface.

/// Viewport/scissor rectangle applied during rendering.
#[derive(Debug, Clone, PartialEq)]
pub struct Viewport {
  pub x: u32,
  pub y: u32,
  pub width: u32,
  pub height: u32,
  pub min_depth: f32,
  pub max_depth: f32,
}

impl Viewport {
  pub(crate) fn viewport_f32(&self) -> (f32, f32, f32, f32, f32, f32) {
    (
      self.x as f32,
      self.y as f32,
      self.width as f32,
      self.height as f32,
      self.min_depth,
      self.max_depth,
    )
  }

  pub(crate) fn scissor_u32(&self) -> (u32, u32, u32, u32) {
    (self.x, self.y, self.width, self.height)
  }
}

/// Builder for viewports used within a render pass.
pub struct ViewportBuilder {
  x: i32,
  y: i32,
  min_depth: f32,
  max_depth: f32,
}

impl Default for ViewportBuilder {
  fn default() -> Self {
    return Self::new();
  }
}

impl ViewportBuilder {
  /// Creates a new viewport builder.
  pub fn new() -> Self {
    Self {
      x: 0,
      y: 0,
      min_depth: 0.0,
      max_depth: 1.0,
    }
  }

  /// Set the top‑left coordinates for the viewport and scissor.
  pub fn with_coordinates(mut self, x: i32, y: i32) -> Self {
    self.x = x;
    self.y = y;
    self
  }

  /// Builds a viewport.
  pub fn build(self, width: u32, height: u32) -> Viewport {
    Viewport {
      x: self.x.max(0) as u32,
      y: self.y.max(0) as u32,
      width,
      height,
      min_depth: self.min_depth,
      max_depth: self.max_depth,
    }
  }
}
