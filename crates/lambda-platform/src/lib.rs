cfg_if::cfg_if! {
  if #[cfg(feature = "with-gl")] {
    pub use gfx_backend_gl as RenderingAPI;
  } else if #[cfg(feature = "with-vulkan")] {
    pub use gfx_backend_vulkan as RenderingAPI;
  } else if #[cfg(feature = "with-metal")] {
    pub use gfx_backend_vulkan as RenderingAPI;
  } else if #[cfg(feature = "with-dx11")] {
    pub use gfx_backend_vulkan as RenderingAPI;
  } else if #[cfg(feature = "with-dx12")] {
    pub use gfx_backend_vulkan as RenderingAPI;
  } else {
    pub use gfx_backend_gl as RenderingAPI;
  }
}
