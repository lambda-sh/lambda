#![allow(clippy::needless_return)]
//! Cross‑platform abstractions and utilities used by Lambda.
//!
//! This crate hosts thin wrappers around `winit` (windowing) and `wgpu`
//! (graphics) that provide consistent defaults and ergonomic builders, along
//! with shader compilation backends and small helper modules (e.g., OBJ
//! loading and random number generation).
pub mod obj;
pub mod rand;
pub mod shader;
pub mod shaderc;
#[cfg(feature = "wgpu")]
pub mod wgpu;
pub mod winit;
