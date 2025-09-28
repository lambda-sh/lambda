//! Runtime implementations and builders for Lambda applications.
//!\n//! This module currently exposes the windowed `ApplicationRuntime` which pairs
//! a `RenderContext` with an event loop and a component stack.
pub mod application;
pub use application::{ApplicationRuntime, ApplicationRuntimeBuilder};
