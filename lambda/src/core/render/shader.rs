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

  pub fn with_name(self, name: &str) -> Self {
    return Self {
      name: Some(String::from(name)),
      shader_source: self.shader_source,
      file_path: self.file_path,
      entry: self.entry,
    };
  }

  pub fn with_shader_source(self, shader_source: &str) -> Self {
    return Self {
      name: self.name,
      shader_source: Some(String::from(shader_source)),
      file_path: self.file_path,
      entry: self.entry,
    };
  }
}

pub struct LambdaShader {
  binary: Vec<u32>,
  kind: ShaderKind,
  metadata: Option<ShaderMetadata>,
}

impl LambdaShader {
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
  pub fn from_file(path: &str, kind: ShaderKind) {
    let mut compiler = shaderc::ShaderCompiler::new();
    let shader_binary = compiler.compile_file_into_binary(path, kind);
  }

	pub fn get_shader_binary(&self) -> &Vec<u32> {
		return &self.binary;
	}

	
}
