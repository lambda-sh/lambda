/// Lambda specific events.
pub enum LambdaEvents {
  Attach,
  Detach,
}

/// Generic Event Enum
pub enum Event {
  Lambda(LambdaEvents),
  Initialized,
  Shutdown,
  Resized { new_width: u32, new_height: u32 },
}
