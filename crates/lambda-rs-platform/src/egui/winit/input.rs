//! This module contains the code to convert winit input events to egui
//! input events.

use egui::PointerButton;
use winit::event::{
  MouseButton,
  VirtualKeyCode,
};

/// Convert a winit mouse button to an egui mouse button.
pub fn winit_to_egui_mouse_button(
  button: MouseButton,
) -> Option<PointerButton> {
  return match button {
    MouseButton::Left => Some(PointerButton::Primary),
    MouseButton::Right => Some(PointerButton::Secondary),
    MouseButton::Middle => Some(PointerButton::Middle),
    MouseButton::Other(1) => Some(PointerButton::Extra1),
    MouseButton::Other(2) => Some(PointerButton::Extra2),
    MouseButton::Other(_) => None,
  };
}

/// Convert a winit virtual key code to an egui key code.
pub fn winit_to_egui_key(key: VirtualKeyCode) -> Option<egui::Key> {
  return Some(match key {
    VirtualKeyCode::Down => egui::Key::ArrowDown,
    VirtualKeyCode::Left => egui::Key::ArrowLeft,
    VirtualKeyCode::Right => egui::Key::ArrowRight,
    VirtualKeyCode::Up => egui::Key::ArrowUp,
    VirtualKeyCode::Escape => egui::Key::Escape,
    VirtualKeyCode::Tab => egui::Key::Tab,
    VirtualKeyCode::Back => egui::Key::Backspace,
    VirtualKeyCode::Return => egui::Key::Enter,
    VirtualKeyCode::Space => egui::Key::Space,
    VirtualKeyCode::Insert => egui::Key::Insert,
    VirtualKeyCode::Delete => egui::Key::Delete,
    VirtualKeyCode::Home => egui::Key::Home,
    VirtualKeyCode::End => egui::Key::End,
    VirtualKeyCode::PageUp => egui::Key::PageUp,
    VirtualKeyCode::PageDown => egui::Key::PageDown,
    VirtualKeyCode::Minus => egui::Key::Minus,
    VirtualKeyCode::Equals => egui::Key::PlusEquals,
    VirtualKeyCode::Key0 | VirtualKeyCode::Numpad0 => egui::Key::Num0,
    VirtualKeyCode::Key1 | VirtualKeyCode::Numpad1 => egui::Key::Num1,
    VirtualKeyCode::Key2 | VirtualKeyCode::Numpad2 => egui::Key::Num2,
    VirtualKeyCode::Key3 | VirtualKeyCode::Numpad3 => egui::Key::Num3,
    VirtualKeyCode::Key4 | VirtualKeyCode::Numpad4 => egui::Key::Num4,
    VirtualKeyCode::Key5 | VirtualKeyCode::Numpad5 => egui::Key::Num5,
    VirtualKeyCode::Key6 | VirtualKeyCode::Numpad6 => egui::Key::Num6,
    VirtualKeyCode::Key7 | VirtualKeyCode::Numpad7 => egui::Key::Num7,
    VirtualKeyCode::Key8 | VirtualKeyCode::Numpad8 => egui::Key::Num8,
    VirtualKeyCode::Key9 | VirtualKeyCode::Numpad9 => egui::Key::Num9,
    VirtualKeyCode::A => egui::Key::A,
    VirtualKeyCode::B => egui::Key::B,
    VirtualKeyCode::C => egui::Key::C,
    VirtualKeyCode::D => egui::Key::D,
    VirtualKeyCode::E => egui::Key::E,
    VirtualKeyCode::F => egui::Key::F,
    VirtualKeyCode::G => egui::Key::G,
    VirtualKeyCode::H => egui::Key::H,
    VirtualKeyCode::I => egui::Key::I,
    VirtualKeyCode::J => egui::Key::J,
    VirtualKeyCode::K => egui::Key::K,
    VirtualKeyCode::L => egui::Key::L,
    VirtualKeyCode::M => egui::Key::M,
    VirtualKeyCode::N => egui::Key::N,
    VirtualKeyCode::O => egui::Key::O,
    VirtualKeyCode::P => egui::Key::P,
    VirtualKeyCode::Q => egui::Key::Q,
    VirtualKeyCode::R => egui::Key::R,
    VirtualKeyCode::S => egui::Key::S,
    VirtualKeyCode::T => egui::Key::T,
    VirtualKeyCode::U => egui::Key::U,
    VirtualKeyCode::V => egui::Key::V,
    VirtualKeyCode::W => egui::Key::W,
    VirtualKeyCode::X => egui::Key::X,
    VirtualKeyCode::Y => egui::Key::Y,
    VirtualKeyCode::Z => egui::Key::Z,
    VirtualKeyCode::F1 => egui::Key::F1,
    VirtualKeyCode::F2 => egui::Key::F2,
    VirtualKeyCode::F3 => egui::Key::F3,
    VirtualKeyCode::F4 => egui::Key::F4,
    VirtualKeyCode::F5 => egui::Key::F5,
    VirtualKeyCode::F6 => egui::Key::F6,
    VirtualKeyCode::F7 => egui::Key::F7,
    VirtualKeyCode::F8 => egui::Key::F8,
    VirtualKeyCode::F9 => egui::Key::F9,
    VirtualKeyCode::F10 => egui::Key::F10,
    VirtualKeyCode::F11 => egui::Key::F11,
    VirtualKeyCode::F12 => egui::Key::F12,
    VirtualKeyCode::F13 => egui::Key::F13,
    VirtualKeyCode::F14 => egui::Key::F14,
    VirtualKeyCode::F15 => egui::Key::F15,
    VirtualKeyCode::F16 => egui::Key::F16,
    VirtualKeyCode::F17 => egui::Key::F17,
    VirtualKeyCode::F18 => egui::Key::F18,
    VirtualKeyCode::F19 => egui::Key::F19,
    VirtualKeyCode::F20 => egui::Key::F20,
    _ => {
      return None;
    }
  });
}

use winit::window::CursorIcon as WinitCursorIcon;

/// Convert an egui mouse cursor icon to a winit mouse cursor icon.
pub fn egui_to_winit_mouse_cursor_icon(
  mouse_cursor_icon: egui::CursorIcon,
) -> Option<winit::window::CursorIcon> {
  return match mouse_cursor_icon {
    egui::CursorIcon::None => None,
    egui::CursorIcon::Alias => Some(WinitCursorIcon::Alias),
    egui::CursorIcon::AllScroll => Some(WinitCursorIcon::AllScroll),
    egui::CursorIcon::Cell => Some(WinitCursorIcon::Cell),
    egui::CursorIcon::ContextMenu => Some(WinitCursorIcon::ContextMenu),
    egui::CursorIcon::Copy => Some(WinitCursorIcon::Copy),
    egui::CursorIcon::Crosshair => Some(WinitCursorIcon::Crosshair),
    egui::CursorIcon::Default => Some(WinitCursorIcon::Default),
    egui::CursorIcon::Grab => Some(WinitCursorIcon::Grab),
    egui::CursorIcon::Grabbing => Some(WinitCursorIcon::Grabbing),
    egui::CursorIcon::Help => Some(WinitCursorIcon::Help),
    egui::CursorIcon::Move => Some(WinitCursorIcon::Move),
    egui::CursorIcon::NoDrop => Some(WinitCursorIcon::NoDrop),
    egui::CursorIcon::NotAllowed => Some(WinitCursorIcon::NotAllowed),
    egui::CursorIcon::PointingHand => Some(WinitCursorIcon::Hand),
    egui::CursorIcon::Progress => Some(WinitCursorIcon::Progress),
    egui::CursorIcon::ResizeHorizontal => Some(WinitCursorIcon::EwResize),
    egui::CursorIcon::ResizeNeSw => Some(WinitCursorIcon::NeswResize),
    egui::CursorIcon::ResizeNwSe => Some(WinitCursorIcon::NwseResize),
    egui::CursorIcon::ResizeVertical => Some(WinitCursorIcon::NsResize),
    egui::CursorIcon::ResizeEast => Some(WinitCursorIcon::EResize),
    egui::CursorIcon::ResizeSouthEast => Some(WinitCursorIcon::SeResize),
    egui::CursorIcon::ResizeSouth => Some(WinitCursorIcon::SResize),
    egui::CursorIcon::ResizeSouthWest => Some(WinitCursorIcon::SwResize),
    egui::CursorIcon::ResizeWest => Some(WinitCursorIcon::WResize),
    egui::CursorIcon::ResizeNorthWest => Some(WinitCursorIcon::NwResize),
    egui::CursorIcon::ResizeNorth => Some(WinitCursorIcon::NResize),
    egui::CursorIcon::ResizeNorthEast => Some(WinitCursorIcon::NeResize),
    egui::CursorIcon::ResizeColumn => Some(WinitCursorIcon::ColResize),
    egui::CursorIcon::ResizeRow => Some(WinitCursorIcon::RowResize),
    egui::CursorIcon::Text => Some(WinitCursorIcon::Text),
    egui::CursorIcon::VerticalText => Some(WinitCursorIcon::VerticalText),
    egui::CursorIcon::Wait => Some(WinitCursorIcon::Wait),
    egui::CursorIcon::ZoomIn => Some(WinitCursorIcon::ZoomIn),
    egui::CursorIcon::ZoomOut => Some(WinitCursorIcon::ZoomOut),
  };
}

/// Check if the keyboard event is a cut event.
pub fn is_keyboard_cut(
  modifiers: egui::Modifiers,
  key_code: winit::event::VirtualKeyCode,
) -> bool {
  let is_cut = modifiers.command && key_code == winit::event::VirtualKeyCode::X;

  let is_cut_with_delete = cfg!(any(
    target_os = "windows",
    all(unix, not(target_os = "macos"))
  )) && modifiers.ctrl
    && key_code == winit::event::VirtualKeyCode::Delete;

  return is_cut || is_cut_with_delete;
}

/// Check if the keyboard event is a copy event.
pub fn is_keyboard_copy(
  modifiers: egui::Modifiers,
  key_code: winit::event::VirtualKeyCode,
) -> bool {
  let is_copy =
    modifiers.command && key_code == winit::event::VirtualKeyCode::C;

  let is_copy_with_insert = cfg!(any(
    target_os = "windows",
    all(unix, not(target_os = "macos"))
  )) && modifiers.ctrl
    && key_code == winit::event::VirtualKeyCode::Insert;

  return is_copy || is_copy_with_insert;
}

/// Check if the keyboard event is a paste event.
pub fn is_keyboard_paste(
  modifiers: egui::Modifiers,
  key_code: winit::event::VirtualKeyCode,
) -> bool {
  let is_paste =
    modifiers.command && key_code == winit::event::VirtualKeyCode::V;

  let is_paste_with_insert = cfg!(any(
    target_os = "windows",
    all(unix, not(target_os = "macos"))
  )) && modifiers.shift
    && key_code == winit::event::VirtualKeyCode::Insert;

  return is_paste || is_paste_with_insert;
}
