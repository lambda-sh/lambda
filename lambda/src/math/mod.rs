#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Triangle {
  color: [f32; 4],
  pos: [f32; 2],
  scale: [f32; 2],
}
