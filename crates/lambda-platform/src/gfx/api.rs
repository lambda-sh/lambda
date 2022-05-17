cfg_if::cfg_if! {
  if #[cfg(feature = "with-gl")] {
    pub use gfx_backend_gl as RenderingAPI;
  } else if #[cfg(feature = "with-vulkan")] {
    pub use gfx_backend_vulkan as RenderingAPI;
  } else if #[cfg(feature = "with-metal")] {
    pub use gfx_backend_metal as RenderingAPI;
  } else if #[cfg(feature = "with-dx11")] {
    pub use gfx_backend_dx11 as RenderingAPI;
  } else if #[cfg(feature = "with-dx12")] {
    pub use gfx_backend_dx12 as RenderingAPI;
  } else if #[cfg(feature = "detect-platform")] {
      pub use gfx_platform_backend as RenderingAPI;
  } else {
  }
}
