/// This module contains a collection of functions translating events and keycodes between SDL2 and [egui].
use egui::Key;
use sdl2::{
    keyboard::{Keycode, Mod},
    mouse::{MouseButton, SystemCursor},
};

pub fn sdl2_to_egui_pointerbutton(button: sdl2::mouse::MouseButton) -> egui::PointerButton {
    match button {
        MouseButton::Left => egui::PointerButton::Primary,
        MouseButton::Right => egui::PointerButton::Secondary,
        MouseButton::Middle => egui::PointerButton::Middle,
        _ => egui::PointerButton::Middle,
    }
}

pub fn sdl2_to_egui_key(
    button: sdl2::keyboard::Keycode,
    mods: sdl2::keyboard::Mod,
    pressed: bool,
) -> Option<egui::Event> {
    let key = match button {
        Keycode::Backspace => Key::Backspace,
        Keycode::Tab => Key::Tab,
        Keycode::Return => Key::Enter,
        Keycode::Escape => Key::Escape,
        Keycode::Space => Key::Space,
        Keycode::Num0 | Keycode::Kp0 => Key::Num0,
        Keycode::Num1 | Keycode::Kp1 => Key::Num1,
        Keycode::Num2 | Keycode::Kp2 => Key::Num2,
        Keycode::Num3 | Keycode::Kp3 => Key::Num3,
        Keycode::Num4 | Keycode::Kp4 => Key::Num4,
        Keycode::Num5 | Keycode::Kp5 => Key::Num5,
        Keycode::Num6 | Keycode::Kp6 => Key::Num6,
        Keycode::Num7 | Keycode::Kp7 => Key::Num7,
        Keycode::Num8 | Keycode::Kp8 => Key::Num8,
        Keycode::Num9 | Keycode::Kp9 => Key::Num9,
        Keycode::Backslash => Key::Backspace,
        Keycode::A => Key::A,
        Keycode::B => Key::B,
        Keycode::C => Key::C,
        Keycode::D => Key::D,
        Keycode::E => Key::E,
        Keycode::F => Key::F,
        Keycode::G => Key::G,
        Keycode::H => Key::H,
        Keycode::I => Key::I,
        Keycode::J => Key::J,
        Keycode::K => Key::K,
        Keycode::L => Key::L,
        Keycode::M => Key::M,
        Keycode::N => Key::N,
        Keycode::O => Key::O,
        Keycode::P => Key::P,
        Keycode::Q => Key::Q,
        Keycode::R => Key::R,
        Keycode::S => Key::S,
        Keycode::T => Key::T,
        Keycode::U => Key::U,
        Keycode::V => Key::V,
        Keycode::W => Key::W,
        Keycode::X => Key::X,
        Keycode::Y => Key::Y,
        Keycode::Z => Key::Z,
        Keycode::Delete => Key::Delete,
        Keycode::Insert => Key::Insert,
        Keycode::Home => Key::Home,
        Keycode::PageUp => Key::PageUp,
        Keycode::End => Key::End,
        Keycode::PageDown => Key::PageDown,
        Keycode::Right => Key::ArrowRight,
        Keycode::Left => Key::ArrowLeft,
        Keycode::Down => Key::ArrowDown,
        Keycode::Up => Key::ArrowUp,
        Keycode::KpEquals => Key::Enter,
        _ => return None,
    };

    let modifiers = egui::Modifiers {
        alt: mods.contains(Mod::LALTMOD | Mod::RALTMOD),
        ctrl: mods.contains(Mod::LCTRLMOD | Mod::RCTRLMOD),
        shift: mods.contains(Mod::LSHIFTMOD | Mod::RSHIFTMOD),
        mac_cmd: false,
        command: false,
    };

    Some(egui::Event::Key {
        key,
        pressed,
        modifiers,
    })
}

pub fn sdl2_to_egui_text(
    button: sdl2::keyboard::Keycode,
    mods: sdl2::keyboard::Mod,
) -> Option<egui::Event> {
    let text = match button {
        Keycode::Space => " ",
        Keycode::Exclaim => "!",
        Keycode::Hash => "#",
        Keycode::Dollar => "$",
        Keycode::Percent => "%",
        Keycode::Ampersand => "&",
        Keycode::Quote => "\"",
        Keycode::LeftParen => "(",
        Keycode::RightParen => ")",
        Keycode::Asterisk => "*",
        Keycode::Plus => "+",
        Keycode::Comma => ",",
        Keycode::Minus => "-",
        Keycode::Period => ".",
        Keycode::Slash => "/",
        Keycode::Num0 | Keycode::Kp0 => "0",
        Keycode::Num1 | Keycode::Kp1 => "1",
        Keycode::Num2 | Keycode::Kp2 => "2",
        Keycode::Num3 | Keycode::Kp3 => "3",
        Keycode::Num4 | Keycode::Kp4 => "4",
        Keycode::Num5 | Keycode::Kp5 => "5",
        Keycode::Num6 | Keycode::Kp6 => "6",
        Keycode::Num7 | Keycode::Kp7 => "7",
        Keycode::Num8 | Keycode::Kp8 => "8",
        Keycode::Num9 | Keycode::Kp9 => "9",
        Keycode::Colon => ":",
        Keycode::Semicolon => ";",
        Keycode::Less => "<",
        Keycode::Equals => "=",
        Keycode::Greater => ">",
        Keycode::Question => "?",
        Keycode::At => "@",
        Keycode::LeftBracket => "[",
        Keycode::Backslash => "\\",
        Keycode::RightBracket => "]",
        Keycode::Caret => "|",
        Keycode::Underscore => "_",
        Keycode::Backquote => "`",
        Keycode::A => "a",
        Keycode::B => "b",
        Keycode::C => "c",
        Keycode::D => "d",
        Keycode::E => "e",
        Keycode::F => "f",
        Keycode::G => "g",
        Keycode::H => "h",
        Keycode::I => "i",
        Keycode::J => "j",
        Keycode::K => "k",
        Keycode::L => "l",
        Keycode::M => "m",
        Keycode::N => "n",
        Keycode::O => "o",
        Keycode::P => "p",
        Keycode::Q => "q",
        Keycode::R => "r",
        Keycode::S => "s",
        Keycode::T => "t",
        Keycode::U => "u",
        Keycode::V => "v",
        Keycode::W => "w",
        Keycode::X => "x",
        Keycode::Y => "y",
        Keycode::Z => "z",
        Keycode::KpDivide => "/",
        Keycode::KpMultiply => "*",
        Keycode::KpMinus => "-",
        Keycode::KpPlus => "+",
        Keycode::KpPeriod => ".",
        Keycode::KpLeftParen => "(",
        Keycode::KpRightParen => ")",
        Keycode::KpLeftBrace => "[",
        Keycode::KpRightBrace => "]",
        _ => return None,
    };

    let uppercase = mods.contains(Mod::LSHIFTMOD | Mod::RSHIFTMOD | Mod::CAPSMOD);
    match uppercase {
        true => Some(egui::Event::Text(text.to_uppercase())),
        false => Some(egui::Event::Text(text.to_owned())),
    }
}

pub fn egui_to_sdl2_cursor(icon: egui::CursorIcon) -> SystemCursor {
    match icon {
        egui::CursorIcon::PointingHand
        | egui::CursorIcon::Grab
        | egui::CursorIcon::Grabbing
        | egui::CursorIcon::Move => SystemCursor::Hand,
        egui::CursorIcon::None | egui::CursorIcon::NotAllowed => SystemCursor::No,
        egui::CursorIcon::Crosshair | egui::CursorIcon::AllScroll => SystemCursor::Crosshair,
        egui::CursorIcon::ResizeHorizontal => SystemCursor::SizeWE,
        egui::CursorIcon::ResizeNeSw => SystemCursor::SizeNESW,
        egui::CursorIcon::ResizeNwSe => SystemCursor::SizeNWSE,
        egui::CursorIcon::ResizeVertical => SystemCursor::SizeNS,
        egui::CursorIcon::Text | egui::CursorIcon::VerticalText => SystemCursor::IBeam,
        egui::CursorIcon::Wait | egui::CursorIcon::Progress => SystemCursor::Wait,
        egui::CursorIcon::ZoomIn | egui::CursorIcon::ZoomOut => SystemCursor::SizeNS,
        _ => SystemCursor::Arrow,
    }
}
