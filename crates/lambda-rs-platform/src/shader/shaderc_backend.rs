use std::io::Read;

use shaderc;

use super::{
  ShaderKind,
  VirtualShader,
};

/// Builder for the shaderc platform shader compiler.
pub struct ShaderCompilerBuilder {}

impl ShaderCompilerBuilder {
  pub fn new() -> Self {
    Self {}
  }

  pub fn build(self) -> ShaderCompiler {
    let compiler =
      shaderc::Compiler::new().expect("Failed to create shaderc compiler.");

    let options = shaderc::CompileOptions::new()
      .expect("Failed to create shaderc compile options.");

    ShaderCompiler {
      compiler,
      default_options: options,
    }
  }
}

/// A low level shader compiler to be used for compiling shaders into SPIR-V binary.
pub struct ShaderCompiler {
  compiler: shaderc::Compiler,
  default_options: shaderc::CompileOptions<'static>,
}

impl ShaderCompiler {
  pub fn compile_into_binary(&mut self, shader: &VirtualShader) -> Vec<u32> {
    match shader {
      VirtualShader::File {
        path,
        kind,
        name,
        entry_point,
      } => self.compile_file_into_binary(
        path.as_str(),
        name.as_str(),
        entry_point.as_str(),
        *kind,
      ),
      VirtualShader::Source {
        source,
        kind,
        name,
        entry_point,
      } => self.compile_string_into_binary(
        source.as_str(),
        name.as_str(),
        entry_point.as_str(),
        *kind,
      ),
    }
  }

  fn compile_file_into_binary(
    &mut self,
    path: &str,
    name: &str,
    entry_point: &str,
    shader_kind: ShaderKind,
  ) -> Vec<u32> {
    let mut opened_shader_file = std::fs::File::open(path).unwrap();
    let mut shader_source = String::new();
    opened_shader_file
      .read_to_string(&mut shader_source)
      .unwrap();

    let compiled_shader = self
      .compiler
      .compile_into_spirv(
        &shader_source,
        map_shader_kind(shader_kind),
        path,
        entry_point,
        Some(&self.default_options),
      )
      .expect("Failed to compile the shader.");
    compiled_shader.as_binary().to_vec()
  }

  fn compile_string_into_binary(
    &mut self,
    shader_source: &str,
    name: &str,
    entry_point: &str,
    shader_kind: ShaderKind,
  ) -> Vec<u32> {
    let compiled_shader = self
      .compiler
      .compile_into_spirv(
        shader_source,
        map_shader_kind(shader_kind),
        name,
        entry_point,
        Some(&self.default_options),
      )
      .expect("Failed to compile the shader.");

    compiled_shader.as_binary().to_vec()
  }
}

fn map_shader_kind(kind: ShaderKind) -> shaderc::ShaderKind {
  match kind {
    ShaderKind::Vertex => shaderc::ShaderKind::Vertex,
    ShaderKind::Fragment => shaderc::ShaderKind::Fragment,
    ShaderKind::Compute => shaderc::ShaderKind::Compute,
  }
}
