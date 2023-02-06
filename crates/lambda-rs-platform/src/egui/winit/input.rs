//! This module contains the code to convert winit input events to egui
//! input events.

use egui::PointerButton;
use winit::event::{
  MouseButton,
  VirtualKeyCode as WinitKey,
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
pub fn winit_to_egui_key(key: WinitKey) -> Option<egui::Key> {
  return Some(match key {
    WinitKey::Down => egui::Key::ArrowDown,
    WinitKey::Left => egui::Key::ArrowLeft,
    WinitKey::Right => egui::Key::ArrowRight,
    WinitKey::Up => egui::Key::ArrowUp,
    WinitKey::Escape => egui::Key::Escape,
    WinitKey::Tab => egui::Key::Tab,
    WinitKey::Back => egui::Key::Backspace,
    WinitKey::Return => egui::Key::Enter,
    WinitKey::Space => egui::Key::Space,
    WinitKey::Insert => egui::Key::Insert,
    WinitKey::Delete => egui::Key::Delete,
    WinitKey::Home => egui::Key::Home,
    WinitKey::End => egui::Key::End,
    WinitKey::PageUp => egui::Key::PageUp,
    WinitKey::PageDown => egui::Key::PageDown,
    WinitKey::Minus => egui::Key::Minus,
    WinitKey::Equals => egui::Key::PlusEquals,
    WinitKey::Key0 | WinitKey::Numpad0 => egui::Key::Num0,
    WinitKey::Key1 | WinitKey::Numpad1 => egui::Key::Num1,
    WinitKey::Key2 | WinitKey::Numpad2 => egui::Key::Num2,
    WinitKey::Key3 | WinitKey::Numpad3 => egui::Key::Num3,
    WinitKey::Key4 | WinitKey::Numpad4 => egui::Key::Num4,
    WinitKey::Key5 | WinitKey::Numpad5 => egui::Key::Num5,
    WinitKey::Key6 | WinitKey::Numpad6 => egui::Key::Num6,
    WinitKey::Key7 | WinitKey::Numpad7 => egui::Key::Num7,
    WinitKey::Key8 | WinitKey::Numpad8 => egui::Key::Num8,
    WinitKey::Key9 | WinitKey::Numpad9 => egui::Key::Num9,
    WinitKey::A => egui::Key::A,
    WinitKey::B => egui::Key::B,
    WinitKey::C => egui::Key::C,
    WinitKey::D => egui::Key::D,
    WinitKey::E => egui::Key::E,
    WinitKey::F => egui::Key::F,
    WinitKey::G => egui::Key::G,
    WinitKey::H => egui::Key::H,
    WinitKey::I => egui::Key::I,
    WinitKey::J => egui::Key::J,
    WinitKey::K => egui::Key::K,
    WinitKey::L => egui::Key::L,
    WinitKey::M => egui::Key::M,
    WinitKey::N => egui::Key::N,
    WinitKey::O => egui::Key::O,
    WinitKey::P => egui::Key::P,
    WinitKey::Q => egui::Key::Q,
    WinitKey::R => egui::Key::R,
    WinitKey::S => egui::Key::S,
    WinitKey::T => egui::Key::T,
    WinitKey::U => egui::Key::U,
    WinitKey::V => egui::Key::V,
    WinitKey::W => egui::Key::W,
    WinitKey::X => egui::Key::X,
    WinitKey::Y => egui::Key::Y,
    WinitKey::Z => egui::Key::Z,
    WinitKey::F1 => egui::Key::F1,
    WinitKey::F2 => egui::Key::F2,
    WinitKey::F3 => egui::Key::F3,
    WinitKey::F4 => egui::Key::F4,
    WinitKey::F5 => egui::Key::F5,
    WinitKey::F6 => egui::Key::F6,
    WinitKey::F7 => egui::Key::F7,
    WinitKey::F8 => egui::Key::F8,
    WinitKey::F9 => egui::Key::F9,
    WinitKey::F10 => egui::Key::F10,
    WinitKey::F11 => egui::Key::F11,
    WinitKey::F12 => egui::Key::F12,
    WinitKey::F13 => egui::Key::F13,
    WinitKey::F14 => egui::Key::F14,
    WinitKey::F15 => egui::Key::F15,
    WinitKey::F16 => egui::Key::F16,
    WinitKey::F17 => egui::Key::F17,
    WinitKey::F18 => egui::Key::F18,
    WinitKey::F19 => egui::Key::F19,
    WinitKey::F20 => egui::Key::F20,
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
pub fn is_keyboard_cut(modifiers: egui::Modifiers, key_code: WinitKey) -> bool {
  let is_cut = modifiers.command && key_code == WinitKey::X;

  let is_cut_with_delete = cfg!(any(
    target_os = "windows",
    all(unix, not(target_os = "macos"))
  )) && modifiers.ctrl
    && key_code == WinitKey::Delete;

  return is_cut || is_cut_with_delete;
}

/// Check if the keyboard event is a copy event.
pub fn is_keyboard_copy(
  modifiers: egui::Modifiers,
  key_code: WinitKey,
) -> bool {
  let is_copy = modifiers.command && key_code == WinitKey::C;

  let is_copy_with_insert = cfg!(any(
    target_os = "windows",
    all(unix, not(target_os = "macos"))
  )) && modifiers.ctrl
    && key_code == WinitKey::Insert;

  return is_copy || is_copy_with_insert;
}

/// Check if the keyboard event is a paste event.
pub fn is_keyboard_paste(
  modifiers: egui::Modifiers,
  key_code: WinitKey,
) -> bool {
  let is_paste = modifiers.command && key_code == WinitKey::V;

  let is_paste_with_insert = cfg!(any(
    target_os = "windows",
    all(unix, not(target_os = "macos"))
  )) && modifiers.shift
    && key_code == WinitKey::Insert;

  return is_paste || is_paste_with_insert;
}
