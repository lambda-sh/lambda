/// Lambda specific events.
pub enum LambdaEvents {
  Attach,
  Detach,
}

pub enum ComponentEvent {
  Attached { name: String },
  Detached { name: String },
}

/// Generic Event Enum
pub enum Event {
  Lambda(LambdaEvents),
  ComponentEvent(ComponentEvent),
  Initialized,
  Shutdown,
  Resized { new_width: u32, new_height: u32 },
}
