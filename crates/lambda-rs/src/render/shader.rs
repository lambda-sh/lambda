//! Shader compilation to SPIR‑V modules.
//!
//! Purpose
//! - Provide a reusable `ShaderBuilder` that turns a `VirtualShader` (inline
//!   GLSL source or file path + metadata) into a SPIR‑V binary suitable for
//!   pipeline creation.
//!
//! Use the platform’s shader backend configured for the workspace (e.g., naga
//! or shaderc) without exposing backend‑specific types in the public API.

// Expose the platform shader compiler abstraction
pub use lambda_platform::shader::{
  ShaderCompiler,
  ShaderCompilerBuilder,
  ShaderKind,
  VirtualShader,
};

/// Reusable compiler for turning virtual shaders into SPIR‑V modules.
///
/// Example
/// ```rust
/// use lambda_platform::shader::{VirtualShader, ShaderKind};
/// use lambda::render::shader::ShaderBuilder;
/// let vs = VirtualShader::File {
///   path: "crates/lambda-rs/assets/shaders/triangle.vert".into(),
///   kind: ShaderKind::Vertex,
///   name: "triangle-vert".into(),
///   entry_point: "main".into(),
/// };
/// let mut builder = ShaderBuilder::new();
/// let vertex_shader = builder.build(vs);
/// ```
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

/// A shader compiled into SPIR‑V binary along with its `VirtualShader` source.
pub struct Shader {
  binary: Vec<u32>,
  virtual_shader: VirtualShader,
}

impl Shader {
  /// Borrow the SPIR‑V binary representation of the shader as a word slice.
  ///
  /// Prefer this accessor to avoid unnecessary allocations when passing the
  /// shader to pipeline builders. Use `as_binary` when an owned buffer is
  /// explicitly required.
  pub fn binary(&self) -> &[u32] {
    return &self.binary;
  }

  /// Returns a copy of the SPIR‑V binary representation of the shader.
  ///
  /// Retained for compatibility with existing code that expects an owned
  /// `Vec<u32>`. Prefer `binary()` for zero‑copy borrowing.
  pub fn as_binary(&self) -> Vec<u32> {
    return self.binary.clone();
  }
}
