//! Shader compilation to SPIR‑V modules.
//!
//! Purpose
//! - Provide a reusable `ShaderBuilder` that turns a `VirtualShader` (inline
//!   GLSL source or file path + metadata) into a SPIR‑V binary suitable for
//!   pipeline creation.
//!
//! Use the platform’s shader backend configured for the workspace (naga)
//! without exposing backend‑specific types in the public API.

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
/// ```rust,no_run
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

impl Default for ShaderBuilder {
  fn default() -> Self {
    return Self::new();
  }
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

  /// Borrow the `VirtualShader` used to compile this shader.
  pub fn virtual_shader(&self) -> &VirtualShader {
    return &self.virtual_shader;
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn shader_builder_compiles_source_and_exposes_binary() {
    let source = r#"
      #version 450
      #extension GL_ARB_separate_shader_objects : enable
      void main() {
        gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
      }
    "#;

    let mut builder = ShaderBuilder::new();
    let shader = builder.build(VirtualShader::Source {
      source: source.to_string(),
      kind: ShaderKind::Vertex,
      name: "test-vert".to_string(),
      entry_point: "main".to_string(),
    });

    assert!(!shader.binary().is_empty());
    assert_eq!(shader.as_binary(), shader.binary());
    assert_eq!(shader.virtual_shader().name(), "test-vert");
    assert!(matches!(shader.virtual_shader().kind(), ShaderKind::Vertex));
  }

  #[test]
  fn shader_builder_compiles_file_shader() {
    let vert_path = format!(
      "{}/assets/shaders/triangle.vert",
      env!("CARGO_MANIFEST_DIR")
    );

    let mut builder = ShaderBuilder::new();
    let shader = builder.build(VirtualShader::File {
      path: vert_path,
      kind: ShaderKind::Vertex,
      name: "triangle-vert".to_string(),
      entry_point: "main".to_string(),
    });

    assert!(!shader.binary().is_empty());
    assert_eq!(shader.virtual_shader().name(), "triangle-vert");
  }
}
