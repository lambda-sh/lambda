pub mod internal {
  pub use gfx_hal::{
    pso::{
      EntryPoint,
      InputAssemblerDesc,
      Primitive,
      PrimitiveAssemblerDesc,
    },
    Backend,
  };

  #[inline]
  pub fn into_primitive_assembler<'shader, RenderBackend: Backend>(
    primitive_assembler: super::PrimitiveAssembler<'shader, RenderBackend>,
  ) -> PrimitiveAssemblerDesc<'shader, RenderBackend> {
    return primitive_assembler.primitive_assembler;
  }
}

/// PrimitiveAssemblerBuilder for preparing PrimitiveAssemblers to use in the
/// lambda-platform Rendering pipeline.
pub struct PrimitiveAssemblerBuilder {}

impl PrimitiveAssemblerBuilder {
  pub fn new() -> Self {
    return Self {};
  }

  /// Build a primitive assembler given the lambda-platform vertex shader
  /// module.
  pub fn build<'shader, RenderBackend: gfx_hal::Backend>(
    self,
    vertex_shader: &'shader super::shader::ShaderModule<RenderBackend>,
  ) -> PrimitiveAssembler<'shader, RenderBackend> {
    // TODO(vmarcella): The builder should expose more fields for the
    let primitive_assembler = internal::PrimitiveAssemblerDesc::Vertex {
      buffers: &[],
      attributes: &[],
      input_assembler: internal::InputAssemblerDesc::new(
        internal::Primitive::TriangleList,
      ),
      vertex: internal::EntryPoint {
        entry: vertex_shader.entry(),
        module: super::internal::module_for(vertex_shader),
        specialization: vertex_shader.specializations().clone(),
      },
      tessellation: None,
      geometry: None,
    };

    return PrimitiveAssembler::<'shader> {
      primitive_assembler,
    };
  }
}

/// PrimitiveAssembler for used for describing how Vertex Shaders should
/// construct primitives. Each constructed Primitive Assembler should be alive
/// for as long as the shader module that created it is.
pub struct PrimitiveAssembler<'shader, RenderBackend: internal::Backend> {
  primitive_assembler: internal::PrimitiveAssemblerDesc<'shader, RenderBackend>,
}

impl<'shader, RenderBackend: internal::Backend>
  PrimitiveAssembler<'shader, RenderBackend>
{
}
