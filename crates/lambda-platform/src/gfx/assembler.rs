//! Primitive assembly for the graphics pipeline.

use gfx_hal::pso;

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
    let primitive_assembler = pso::PrimitiveAssemblerDesc::Vertex {
      buffers: &[],
      attributes: &[],
      input_assembler: pso::InputAssemblerDesc::new(
        pso::Primitive::TriangleList,
      ),
      vertex: pso::EntryPoint {
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
pub struct PrimitiveAssembler<'shader, RenderBackend: gfx_hal::Backend> {
  primitive_assembler: pso::PrimitiveAssemblerDesc<'shader, RenderBackend>,
}

impl<'shader, RenderBackend: gfx_hal::Backend>
  PrimitiveAssembler<'shader, RenderBackend>
{
}

/// Internal functions for the primitive assembler. User applications most
/// likely should not use these functions directly nor should they need to.
pub(crate) mod internal {
  #[inline]
  pub fn into_primitive_assembler<'shader, RenderBackend: gfx_hal::Backend>(
    primitive_assembler: super::PrimitiveAssembler<'shader, RenderBackend>,
  ) -> gfx_hal::pso::PrimitiveAssemblerDesc<'shader, RenderBackend> {
    return primitive_assembler.primitive_assembler;
  }
}
