//! Primitive assembly for the graphics pipeline.

pub use gfx_hal::pso::Element as VertexElement;
use gfx_hal::pso::{
  self,
  AttributeDesc,
  VertexBufferDesc,
};

use super::{
  buffer::Buffer,
  surface::ColorFormat,
};

/// Attributes for a vertex.
#[derive(Debug, Clone)]
pub struct VertexAttribute {
  pub location: u32,
  pub offset: u32,
  pub element: VertexElement<ColorFormat>,
}

/// PrimitiveAssemblerBuilder for preparing PrimitiveAssemblers to use in the
/// lambda-platform Rendering pipeline.
pub struct PrimitiveAssemblerBuilder {
  buffer_descriptions: Vec<VertexBufferDesc>,
  attribute_descriptions: Vec<AttributeDesc>,
}

impl PrimitiveAssemblerBuilder {
  pub fn new() -> Self {
    return Self {
      buffer_descriptions: Vec::new(),
      attribute_descriptions: Vec::new(),
    };
  }

  /// Build a primitive assembler given the lambda-platform vertex shader
  /// module. Buffers & attributes do not have to be tied to
  pub fn build<'shader, RenderBackend: gfx_hal::Backend>(
    &'shader mut self,
    vertex_shader: &'shader super::shader::ShaderModule<RenderBackend>,
    buffers: Option<&Vec<&Buffer<RenderBackend>>>,
    attributes: Option<&[VertexAttribute]>,
  ) -> PrimitiveAssembler<'shader, RenderBackend> {
    let binding = self.buffer_descriptions.len() as u32;

    match (buffers, attributes) {
      (Some(buffers), Some(attributes)) => {
        logging::debug!(
          "Building primitive assembler with buffers and attributes"
        );
        self.buffer_descriptions = buffers
          .iter()
          .map(|buffer| VertexBufferDesc {
            binding,
            stride: buffer.stride() as u32,
            rate: pso::VertexInputRate::Vertex,
          })
          .collect();

        self.attribute_descriptions = attributes
          .iter()
          .map(|attribute| {
            return AttributeDesc {
              location: attribute.location,
              binding,
              element: attribute.element,
            };
          })
          .collect();
      }
      _ => {}
    }

    let primitive_assembler = pso::PrimitiveAssemblerDesc::Vertex {
      buffers: self.buffer_descriptions.as_slice(),
      attributes: self.attribute_descriptions.as_slice(),
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
  // Get the internal primitive assembler.
  pub(super) fn internal_primitive_assembler(
    self,
  ) -> pso::PrimitiveAssemblerDesc<'shader, RenderBackend> {
    return self.primitive_assembler;
  }
}
