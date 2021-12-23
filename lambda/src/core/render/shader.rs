use crate::platform::shaderc;

/// Supported Shader kinds.
#[derive(Clone, Copy)]
pub enum ShaderKind {
  Vertex,
  Fragment,
  Compute,
}

/// Optional ShaderMetadata that can be used for creating
struct ShaderMetadata {
  name: Option<String>,
  shader_source: Option<String>,
  file_path: Option<String>,
  entry: Option<String>,
}

impl ShaderMetadata {
  pub fn new() -> Self {
    return Self {
      name: None,
      shader_source: None,
      file_path: None,
      entry: None,
    };
  }

  /// Attach name to the ShaderMetadata.
  pub fn with_name(self, name: &str) -> Self {
    return Self {
      name: Some(String::from(name)),
      shader_source: self.shader_source,
      file_path: self.file_path,
      entry: self.entry,
    };
  }

  /// Attach the shader source code to a LambdaShader
  pub fn with_shader_source(self, shader_source: &str) -> Self {
    return Self {
      name: self.name,
      shader_source: Some(String::from(shader_source)),
      file_path: self.file_path,
      entry: self.entry,
    };
  }
}

pub struct Shader {
  binary: Vec<u32>,
  kind: ShaderKind,
  metadata: Option<ShaderMetadata>,
}

pub enum VertexShaders {
  Triangle,
}

pub enum FragmentShaders {
  Triangle,
}

pub enum PrepackagedShaders {
  Vertex(VertexShaders),
  Fragment(FragmentShaders),
}

impl Shader {
  /// Creates a shader given a source string.
  pub fn from_string(name: &str, source: &str, kind: ShaderKind) -> Self {
    let mut compiler = shaderc::ShaderCompiler::new();
    let shader_binary = compiler.compile_string_into_binary(name, source, kind);

    return Self {
      binary: shader_binary,
      kind,
      metadata: None,
    };
  }

  /// Creates a shader given a file path.
  pub fn from_file(path: &str, kind: ShaderKind) -> Self {
    let mut compiler = shaderc::ShaderCompiler::new();
    let shader_binary = compiler.compile_file_into_binary(path, kind);

    return Self {
      binary: shader_binary,
      kind,
      metadata: None,
    };
  }

  pub fn from_lambda(shader: PrepackagedShaders) -> Self {
    let mut compiler = shaderc::ShaderCompiler::new();
    let (asset_path, kind) = resolve_shader_path(shader);
    let shader_binary = compiler.compile_file_into_binary(&asset_path, kind);

    return Self {
      binary: shader_binary,
      kind,
      metadata: None,
    };
  }

  pub fn get_shader_binary(&self) -> &Vec<u32> {
    return &self.binary;
  }
}

pub fn resolve_shader_path(shader: PrepackagedShaders) -> (String, ShaderKind) {
  return match shader {
    PrepackagedShaders::Vertex(shader) => {
      let kind = ShaderKind::Vertex;
      match shader {
        VertexShaders::Triangle => {
          ("assets/shaders/triangle.vert".to_string(), kind)
        }
      }
    }
    PrepackagedShaders::Fragment(shader) => {
      let kind = ShaderKind::Fragment;
      return match shader {
        FragmentShaders::Triangle => {
          ("assets/shaders/triangle.frag".to_string(), kind)
        }
      };
    }
  };
}
