use std::io::Read;

use shaderc;
/// shaderc exports for use throughout
pub use shaderc::ShaderKind;

pub struct ShaderCompiler {
  compiler: shaderc::Compiler,
  default_options: shaderc::CompileOptions<'static>,
}

impl ShaderCompiler {
  pub fn new() -> Self {
    let compiler = shaderc::Compiler::new().unwrap();
    let default_options = shaderc::CompileOptions::new().unwrap();

    return Self {
      compiler,
      default_options,
    };
  }

  /// Compiles a file at the given path into a shader and returns it as binary.
  pub fn compile_file_into_binary(
    &mut self,
    path: &str,
    shader_kind: ShaderKind,
  ) -> Vec<u32> {
    // TODO(vmarcella): Investigate into common strategies for reading from files
    // efficiently in Rust.
    let mut opened_shader_file = std::fs::File::open(path).unwrap();
    let mut shader_source = String::new();
    opened_shader_file
      .read_to_string(&mut shader_source)
      .unwrap();

    // TODO(vmarcella): Should we be allow entrypoints to be customized or
    // enforce that all remain named main?
    let compiled_shader = self
      .compiler
      .compile_into_spirv(&shader_source, shader_kind, path, "main", None)
      .expect("Failed to compile the shader.");
    return compiled_shader.as_binary().to_vec();
  }

  pub fn compile_string_into_binary(
    &self,
    name: &str,
    string: &str,
    shader_kind: ShaderKind,
  ) {
    todo!("ShaderCompiler currently doesn't implement compiling a string into spirv binary.")
  }
}
