use std::time::Instant;

/// events generated by kernel interactions with the component.
#[derive(Debug, Clone)]
pub enum ComponentEvent {
  Attached { name: String },
  Detached { name: String },
}

/// Window events are generated in response to window events coming from
/// the windowing system.
#[derive(Debug, Clone)]
pub enum WindowEvent {
  Close,
  Resize { width: u32, height: u32 },
}

/// Kernel events are generated by the kernel itself
#[derive(Debug, Clone)]
pub enum RuntimeEvent {
  Initialized,
  Shutdown,
}

/// Exports the winit virtual key codes to this namespace for convenience.
pub use lambda_platform::winit::winit_exports::VirtualKeyCode as VirtualKey;

#[derive(Debug, Clone)]
pub enum KeyEvent {
  KeyPressed {
    scan_code: u32,
    virtual_key: Option<VirtualKey>,
  },
  KeyReleased {
    scan_code: u32,
    virtual_key: Option<VirtualKey>,
  },
  ModifierPressed {
    modifier: u32,
    virtual_key: VirtualKey,
  },
}

#[derive(Debug, Clone)]
pub enum Mouse {
  MouseMoved { x: f32, y: f32, dx: f32, dy: f32 },
  MouseWheelPressed { x: f32, y: f32, button: u32 },
  MousePressed { x: f32, y: f32, button: u32 },
  MouseReleased { x: f32, y: f32, button: u32 },
}

/// Generic Event Enum which encapsulates all possible events that will be
/// emitted by the LambdaKernel
#[derive(Debug, Clone)]
pub enum Events {
  Component {
    event: ComponentEvent,
    issued_at: Instant,
  },
  Window {
    event: WindowEvent,
    issued_at: Instant,
  },
  Runtime {
    event: RuntimeEvent,
    issued_at: Instant,
  },
  Keyboard {
    event: KeyEvent,
    issued_at: Instant,
  },
  Mouse {
    event: Mouse,
    issued_at: Instant,
  },
}
