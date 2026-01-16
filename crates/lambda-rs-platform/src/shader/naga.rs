use std::io::Read;

use naga::{
  back::spv,
  front::glsl,
  valid::{
    Capabilities,
    ValidationFlags,
    Validator,
  },
  ShaderStage,
};

use super::{
  ShaderKind,
  VirtualShader,
};

/// Builder for the naga-backed shader compiler.
pub struct ShaderCompilerBuilder {}

impl ShaderCompilerBuilder {
  pub fn new() -> Self {
    Self {}
  }

  pub fn build(self) -> ShaderCompiler {
    ShaderCompiler {}
  }
}

impl Default for ShaderCompilerBuilder {
  fn default() -> Self {
    return Self::new();
  }
}

/// A shader compiler that uses naga to translate shader sources into SPIR-V.
pub struct ShaderCompiler {}

impl ShaderCompiler {
  pub fn compile_into_binary(&mut self, shader: &VirtualShader) -> Vec<u32> {
    match shader {
      VirtualShader::File {
        path,
        kind,
        name,
        entry_point,
      } => {
        let mut file =
          std::fs::File::open(path).expect("Failed to open shader file.");
        let mut shader_source = String::new();
        file
          .read_to_string(&mut shader_source)
          .expect("Failed to read shader file.");

        self.compile_string_into_binary(
          shader_source.as_str(),
          name.as_str(),
          entry_point.as_str(),
          *kind,
        )
      }
      VirtualShader::Source {
        source,
        kind,
        name,
        entry_point,
      } => self.compile_string_into_binary(
        source.as_str(),
        name.as_str(),
        entry_point.as_str(),
        *kind,
      ),
    }
  }

  fn compile_string_into_binary(
    &mut self,
    shader_source: &str,
    name: &str,
    entry_point: &str,
    shader_kind: ShaderKind,
  ) -> Vec<u32> {
    let stage = shader_kind_to_stage(shader_kind);
    let mut frontend = glsl::Frontend::default();
    let module = frontend
      .parse(&glsl::Options::from(stage), shader_source)
      .unwrap_or_else(|err| {
        panic!("Failed to compile shader {}: {:?}", name, err)
      });

    let mut validator =
      Validator::new(ValidationFlags::all(), Capabilities::all());
    let module_info = validator
      .validate(&module)
      .expect("Failed to validate shader module.");

    let options = spv::Options {
      lang_version: (1, 5),
      flags: spv::WriterFlags::empty(),
      ..Default::default()
    };

    let pipeline_options = spv::PipelineOptions {
      shader_stage: stage,
      entry_point: entry_point.to_string(),
    };

    spv::write_vec(&module, &module_info, &options, Some(&pipeline_options))
      .expect("Failed to translate shader module into SPIR-V.")
  }
}

fn shader_kind_to_stage(kind: ShaderKind) -> ShaderStage {
  match kind {
    ShaderKind::Vertex => ShaderStage::Vertex,
    ShaderKind::Fragment => ShaderStage::Fragment,
    ShaderKind::Compute => ShaderStage::Compute,
  }
}
