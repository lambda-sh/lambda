use gfx_hal::pso::EntryPoint;
use gfx_hal::pso::InputAssemblerDesc;
use gfx_hal::Backend;
use gfx_hal::pso::PrimitiveAssemblerDesc;
use gfx_hal::pso::Primitive;

/// Create a primitive vertex assembler with no current configurations.
pub fn create_vertex_assembler<'a, B: Backend>(vertex_entry: EntryPoint<'a, B>) -> PrimitiveAssemblerDesc<'a, B> {
    return PrimitiveAssemblerDesc::Vertex{
        buffers: &[],
        attributes: &[],
        input_assembler: InputAssemblerDesc::new(Primitive::TriangleList),
        vertex: vertex_entry,
        tessellation: None,
        geometry: None,
    }
}
