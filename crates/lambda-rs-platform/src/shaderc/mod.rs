use std::io::Read;

use shaderc;
/// Export supported shader kinds.
pub use shaderc::ShaderKind;

/// Builder for the shaderc platform shader compiler.
pub struct ShaderCompilerBuilder {}

impl ShaderCompilerBuilder {
  pub fn new() -> Self {
    return Self {};
  }

  pub fn build(self) -> ShaderCompiler {
    let compiler =
      shaderc::Compiler::new().expect("Failed to create shaderc compiler.");

    let options = shaderc::CompileOptions::new()
      .expect("Failed to create shaderc compile options.");

    return ShaderCompiler {
      compiler,
      default_options: options,
    };
  }
}

/// A low level shader compiler to be used for compiling shaders into SPIR-V binary.
pub struct ShaderCompiler {
  compiler: shaderc::Compiler,
  default_options: shaderc::CompileOptions<'static>,
}

/// Meta Representations of real shaders to use for easy compilation
#[derive(Debug, Clone)]
pub enum VirtualShader {
  File {
    path: String,
    kind: ShaderKind,
    name: String,
    entry_point: String,
  },
  Source {
    source: String,
    kind: ShaderKind,
    name: String,
    entry_point: String,
  },
}

impl ShaderCompiler {
  /// Compiles a shader into SPIR-V binary.
  pub fn compile_into_binary(&mut self, shader: &VirtualShader) -> Vec<u32> {
    return match shader {
      VirtualShader::File {
        path,
        kind,
        name,
        entry_point,
      } => {
        return self.compile_file_into_binary(
          path.as_str(),
          name.as_str(),
          entry_point.as_str(),
          kind.clone(),
        )
      }
      VirtualShader::Source {
        source,
        kind,
        name,
        entry_point,
      } => self.compile_string_into_binary(
        source.as_str(),
        name.as_str(),
        entry_point.as_str(),
        kind.clone(),
      ),
    };
  }

  /// Compiles a file at the given path into a shader and returns it as binary.
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
        shader_kind,
        path,
        entry_point,
        Some(&self.default_options),
      )
      .expect("Failed to compile the shader.");
    return compiled_shader.as_binary().to_vec();
  }

  // Compiles a string into SPIR-V binary.
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
        shader_kind,
        name,
        entry_point,
        Some(&self.default_options),
      )
      .expect("Failed to compile the shader.");

    return compiled_shader.as_binary().to_vec();
  }
}
