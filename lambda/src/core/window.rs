/// The base window trait that every lambda window implementation must have to
/// work with lambda::core components.
pub trait WindowAPI {
  fn new() -> Self;
  fn redraw(&self);
  fn close(&self);
}
