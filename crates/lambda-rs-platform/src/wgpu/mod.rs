//! Cross‑platform GPU abstraction built on top of `wgpu`.
//!
//! This module exposes a small, opinionated wrapper around core `wgpu` types
//! to make engine code concise while keeping configuration explicit. The
//! builders here (for the instance, surface, and device/queue) provide sane
//! defaults and narrow the surface area used by Lambda, without hiding
//! important handles when you need to drop down to raw `wgpu`.

// keep this module focused on exports and submodules

pub mod bind;
pub mod buffer;
pub mod command;
pub mod gpu;
pub mod instance;
pub mod pipeline;
pub mod render_pass;
pub mod surface;
pub mod vertex;
