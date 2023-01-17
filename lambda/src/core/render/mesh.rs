use super::vertex::Vertex;

pub struct Mesh {
  pub vertices: Vec<Vertex>,
  pub indices: Vec<u32>,
}
