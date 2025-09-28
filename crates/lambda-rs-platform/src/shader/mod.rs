//! Abstractions for compiling shaders into SPIR-V for Lambda runtimes.

mod types;
pub use types::{
  ShaderKind,
  VirtualShader,
};

#[cfg(feature = "shader-backend-naga")]
mod naga;

#[cfg(feature = "shader-backend-shaderc")]
mod shaderc_backend;

#[cfg(feature = "shader-backend-naga")]
pub use naga::{
  ShaderCompiler,
  ShaderCompilerBuilder,
};
#[cfg(all(
  feature = "shader-backend-naga",
  feature = "shader-backend-shaderc"
))]
pub use naga::{
  ShaderCompiler,
  ShaderCompilerBuilder,
};
#[cfg(all(
  not(feature = "shader-backend-naga"),
  feature = "shader-backend-shaderc"
))]
pub use shaderc_backend::{
  ShaderCompiler,
  ShaderCompilerBuilder,
};
