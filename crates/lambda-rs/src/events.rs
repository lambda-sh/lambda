//! Event definitions for lambda runtimes and applications.

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

/// Runtime events are generated by the Runtimes themselves.
#[derive(Debug, Clone)]
pub enum RuntimeEvent {
  Initialized,
  Shutdown,
  ComponentPanic { message: String },
}

/// Exports the winit virtual key codes to this namespace for convenience.
pub use lambda_platform::winit::winit_exports::VirtualKeyCode as VirtualKey;

/// Keyboard events are generated in response to keyboard events coming from
/// the windowing system.
#[derive(Debug, Clone)]
pub enum Key {
  /// Emitted when a key is pressed.
  Pressed {
    scan_code: u32,
    virtual_key: Option<VirtualKey>,
  },
  /// Emitted when a key is released.
  Released {
    scan_code: u32,
    virtual_key: Option<VirtualKey>,
  },
  /// Emitted when a modifier key is pressed.
  ModifierPressed {
    modifier: u32,
    virtual_key: VirtualKey,
  },
}

/// Mouse buttons.
#[derive(Debug, Clone)]
pub enum Button {
  Left,
  Right,
  Middle,
  Other(u16),
}

/// Mouse events are generated in response to mouse events coming from the
/// windowing system. The coordinates are in logical pixels.
#[derive(Debug, Clone)]
pub enum Mouse {
  /// Emitted when the mouse cursor is moved within the window.
  Moved {
    x: f64,
    y: f64,
    dx: f64,
    dy: f64,
    device_id: u32,
  },
  /// Emitted when the mouse wheel is scrolled.
  Scrolled { device_id: u32 },
  /// Emitted when a mouse button is pressed.
  Pressed {
    x: f64,
    y: f64,
    button: Button,
    device_id: u32,
  },
  /// Emitted when a mouse button is released.
  Released {
    x: f64,
    y: f64,
    button: Button,
    device_id: u32,
  },
  /// Emitted when the mouse cursor leaves the window.
  LeftWindow { device_id: u32 },
  /// Emitted when the mouse cursor enters the window.
  EnteredWindow { device_id: u32 },
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
    event: Key,
    issued_at: Instant,
  },
  Mouse {
    event: Mouse,
    issued_at: Instant,
  },
}
