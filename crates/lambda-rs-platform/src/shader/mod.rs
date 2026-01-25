//! Abstractions for compiling shaders into SPIR-V for Lambda runtimes.

mod types;
pub use types::{
  ShaderKind,
  VirtualShader,
};

#[cfg(feature = "shader-backend-naga")]
mod naga;

#[cfg(feature = "shader-backend-naga")]
pub use naga::{
  ShaderCompiler,
  ShaderCompilerBuilder,
};
