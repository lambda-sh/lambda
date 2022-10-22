//! GPU API exports to set the platforms primary rendering API for rendering
//! implementations to use.

cfg_if::cfg_if! {
  if #[cfg(feature = "gfx-with-gl")] {
    pub use gfx_backend_gl as RenderingAPI;
  } else if #[cfg(feature = "gfx-with-vulkan")] {
    pub use gfx_backend_vulkan as RenderingAPI;
  } else if #[cfg(feature = "gfx-with-metal")] {
    pub use gfx_backend_metal as RenderingAPI;
  } else if #[cfg(feature = "gfx-with-dx11")] {
    pub use gfx_backend_dx11 as RenderingAPI;
  } else if #[cfg(feature = "gfx-with-dx12")] {
    pub use gfx_backend_dx12 as RenderingAPI;
  } else if #[cfg(feature = "detect-platform")] {
      pub use gfx_platform_backend as RenderingAPI;
  } else {}
}
