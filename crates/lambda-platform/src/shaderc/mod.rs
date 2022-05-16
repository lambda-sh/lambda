use std::io::Read;

extern crate shaderc as shaderc_lib;

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
    let mut compiler = ShaderCompiler::new();
    let shader_binary = compiler.compile_string_into_binary(name, source, kind);

    return Self {
      binary: shader_binary,
      kind,
      metadata: None,
    };
  }

  /// Creates a shader given a file path.
  pub fn from_file(path: &str, kind: ShaderKind) -> Self {
    let mut compiler = ShaderCompiler::new();
    let shader_binary = compiler.compile_file_into_binary(path, kind);

    return Self {
      binary: shader_binary,
      kind,
      metadata: None,
    };
  }

  pub fn from_lambda(shader: PrepackagedShaders) -> Self {
    let mut compiler = ShaderCompiler::new();
    let (shader_source, kind) = get_shader_source(shader);
    let shader_binary = compiler.compile_string_into_binary(
      "triangle_tests",
      &shader_source,
      kind,
    );

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

pub fn get_shader_source(shader: PrepackagedShaders) -> (String, ShaderKind) {
  // TODO(vmarcella): Shaders should most certainly not be loaded into the
  // library like this.
  return match shader {
    PrepackagedShaders::Vertex(shader) => {
      let kind = ShaderKind::Vertex;
      match shader {
        VertexShaders::Triangle => (
          include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/shaders/triangle.vert"
          ))
          .to_string(),
          kind,
        ),
      }
    }
    PrepackagedShaders::Fragment(shader) => {
      let kind = ShaderKind::Fragment;
      return match shader {
        FragmentShaders::Triangle => (
          include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/shaders/triangle.frag"
          ))
          .to_string(),
          kind,
        ),
      };
    }
  };
}

/// Converts shader::ShaderKind to a corresponding shaderc::ShaderKind
fn shader_to_shaderc(shader_kind: ShaderKind) -> shaderc::ShaderKind {
  return match shader_kind {
    ShaderKind::Vertex => shaderc_lib::ShaderKind::Vertex,
    ShaderKind::Fragment => shaderc_lib::ShaderKind::Fragment,
    ShaderKind::Compute => shaderc_lib::ShaderKind::Compute,
  };
}

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
      .compile_into_spirv(
        &shader_source,
        shader_to_shaderc(shader_kind),
        path,
        "main",
        None,
      )
      .expect("Failed to compile the shader.");
    return compiled_shader.as_binary().to_vec();
  }

  pub fn compile_string_into_binary(
    &mut self,
    name: &str,
    shader_source: &str,
    shader_kind: ShaderKind,
  ) -> Vec<u32> {
    let compiled_shader = self
      .compiler
      .compile_into_spirv(
        shader_source,
        shader_to_shaderc(shader_kind),
        name,
        "main",
        None,
      )
      .expect("Failed to compile the shader.");
    return compiled_shader.as_binary().to_vec();
  }
}
