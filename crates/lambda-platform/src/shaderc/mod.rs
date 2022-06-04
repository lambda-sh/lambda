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
    return ShaderCompiler {
      compiler,
      default_options: shaderc::CompileOptions::new()
        .expect("Failed to set the default shaderc compiler options"),
    };
  }
}

/// A low level shader compiler to be used for compiling shaders into SPIR-V binary.
pub struct ShaderCompiler {
  compiler: shaderc::Compiler,
  default_options: shaderc::CompileOptions<'static>,
}

/// Meta Representations of real shaders to use for easy compilation
pub enum MetaShader {
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
  pub fn compile_into_binary(&mut self, shader: &MetaShader) -> Vec<u32> {
    return match shader {
      MetaShader::File {
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
      MetaShader::Source {
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
    // TODO(vmarcella): Investigate into common strategies for reading from files
    // efficiently in Rust.
    let mut opened_shader_file = std::fs::File::open(path).unwrap();
    let mut shader_source = String::new();
    opened_shader_file
      .read_to_string(&mut shader_source)
      .unwrap();

    let compiled_shader = self
      .compiler
      .compile_into_spirv(&shader_source, shader_kind, path, entry_point, None)
      .expect("Failed to compile the shader.");
    return compiled_shader.as_binary().to_vec();
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
      .compile_into_spirv(shader_source, shader_kind, name, entry_point, None)
      .expect("Failed to compile the shader.");
    return compiled_shader.as_binary().to_vec();
  }
}
