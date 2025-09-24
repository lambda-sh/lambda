//! Shared shader metadata structures used across shader backends.

/// Supported shader stages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderKind {
  Vertex,
  Fragment,
  Compute,
}

/// Meta representations of real shaders to use for easy compilation.
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

impl VirtualShader {
  pub fn kind(&self) -> ShaderKind {
    match self {
      VirtualShader::File { kind, .. } => *kind,
      VirtualShader::Source { kind, .. } => *kind,
    }
  }

  pub fn entry_point(&self) -> &str {
    match self {
      VirtualShader::File { entry_point, .. }
      | VirtualShader::Source { entry_point, .. } => entry_point.as_str(),
    }
  }

  pub fn name(&self) -> &str {
    match self {
      VirtualShader::File { name, .. } | VirtualShader::Source { name, .. } => {
        name.as_str()
      }
    }
  }
}
