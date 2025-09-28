//! A module for compiling shaders into SPIR-V binary.

// Expose the platform shader compiler abstraction
pub use lambda_platform::shader::{
  ShaderCompiler, ShaderCompilerBuilder, ShaderKind, VirtualShader,
};

/// Reusable compiler for turning virtual shaders into SPIR‑V modules.
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
  pub fn build(&mut self, virtual_shader: VirtualShader) -> Shader {
    logging::trace!("Compiling shader: {:?}", virtual_shader);
    let binary = self.compiler.compile_into_binary(&virtual_shader);

    return Shader {
      binary,
      virtual_shader,
    };
  }
}

/// A shader that has been compiled into SPIR-V binary. Contains the binary
/// representation of the shader as well as the virtual shader that was used
/// to compile it.
pub struct Shader {
  binary: Vec<u32>,
  virtual_shader: VirtualShader,
}

impl Shader {
  /// Returns a copy of the SPIR-V binary representation of the shader.
  pub fn as_binary(&self) -> Vec<u32> {
    return self.binary.clone();
  }
}
