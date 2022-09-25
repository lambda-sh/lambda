use std::time::Instant;

pub enum ComponentEvent {
  Attached { name: String },
  Detached { name: String },
}

pub enum WindowEvent {
  Close,
  Resize { width: u32, height: u32 },
}

pub enum KernelEvent {
  Initialized,
  Shutdown,
}

/// Generic Event Enum which encapsulates all possible events that will be emitted
/// by the LambdaKernel
pub enum Events {
  Component {
    event: ComponentEvent,
    issued_at: Instant,
  },
  Window {
    event: WindowEvent,
    issued_at: Instant,
  },
  Kernel {
    event: KernelEvent,
    issued_at: Instant,
  },
}
