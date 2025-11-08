//! Cross‑platform GPU abstraction built on top of `wgpu`.
//!
//! This module exposes a small, opinionated wrapper around core `wgpu` types
//! organized into focused submodules (instance, surface, gpu, pipeline, etc.).
//! Higher layers import these modules rather than raw `wgpu` to keep Lambda’s
//! API compact and stable.

// Keep this module focused on exports and submodules only.
pub mod bind;
pub mod buffer;
pub mod command;
pub mod gpu;
pub mod instance;
pub mod pipeline;
pub mod render_pass;
pub mod surface;
pub mod texture;
pub mod vertex;
