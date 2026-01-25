//! Abstractions for compiling shaders into SPIR-V for Lambda runtimes.

mod types;
pub use types::{
  ShaderKind,
  VirtualShader,
};

mod naga;
pub use naga::{
  ShaderCompiler,
  ShaderCompilerBuilder,
};
