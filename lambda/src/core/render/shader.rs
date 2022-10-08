// Expose some lower level shader
pub use lambda_platform::shaderc::{
  ShaderCompiler,
  ShaderCompilerBuilder,
  ShaderKind,
  VirtualShader,
};

/// Reusable shader builder that utilizes a lower level platform
pub struct ShaderBuilder {
  compiler: ShaderCompiler,
}

impl ShaderBuilder {
  /// Creates a new shader builder that can be reused for compiling shaders.
  pub fn new() -> Self {
    let compiler = ShaderCompilerBuilder::new().build();
    return Self { compiler };
  }

  /// Compiles the virtual shader into a real shader with SPIR-V binary
  /// representation.
  pub fn build(&mut self, meta_shader: VirtualShader) -> Shader {
    let binary = self.compiler.compile_into_binary(&meta_shader);

    return Shader {
      binary,
      virtual_shader: meta_shader,
    };
  }
}

pub struct Shader {
  binary: Vec<u32>,
  virtual_shader: VirtualShader,
}

impl Shader {
  pub fn as_binary(&self) -> Vec<u32> {
    return self.binary.clone();
  }
}
