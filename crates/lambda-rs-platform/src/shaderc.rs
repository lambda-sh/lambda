//! Deprecated re-exports for code that still references the legacy shaderc module.

#[deprecated(
  since = "2023.1.31",
  note = "Use `lambda_platform::shader` instead of `lambda_platform::shaderc`."
)]
pub use crate::shader::{
  ShaderCompiler,
  ShaderCompilerBuilder,
  ShaderKind,
  VirtualShader,
};
