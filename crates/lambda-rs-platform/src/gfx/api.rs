//! GPU API exports to set the platforms primary rendering API for rendering
//! implementations to use.

cfg_if::cfg_if! {
if #[cfg(any(feature = "gfx-with-gl", all(feature = "detect-platform", unix, not(target_os="macos")) ))] {
  pub use gfx_backend_gl as RenderingAPI;
} else if #[cfg(any(feature = "gfx-with-metal", all(feature = "detect-platform", target_os="macos")))] {
  pub use gfx_backend_metal as RenderingAPI;
} else if #[cfg(feature = "gfx-with-vulkan")] {
  pub use gfx_backend_vulkan as RenderingAPI;
} else if #[cfg(feature = "gfx-with-dx11")] {
  pub use gfx_backend_dx11 as RenderingAPI;
} else if #[cfg(any(feature = "gfx-with-dx12", all(windows, feature = "detect-platform")))] {
  pub use gfx_backend_dx12 as RenderingAPI;
} else {
    pub use gfx_backend_empty as RenderingAPI;
    println!("[WARN] No rendering backend specified, using empty backend.");
  }
}
