use std::borrow::Cow;

use crate::keys::{self, KeyRecord};
use crate::layout::{self, KeyGeom};

/// Dirección de una flecha de la capa Fn. No es un carácter Unicode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    Up,
    Left,
    Down,
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mark {
    Text(Cow<'static, str>),
    Arrow(Dir),
}

/// Lo que se dibuja en una tecla.
///
/// `hint` es la tecla física, y solo aparece en la capa Fn cuando la función
/// es otra cosa. Nunca es el genérico "Tecla".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Caption {
    pub mark: Mark,
    pub hint: Option<&'static str>,
}

pub fn caption(fn_layer: bool, key: &KeyGeom, record: KeyRecord) -> Caption {
    match resolve(record) {
        Some(Mark::Arrow(dir)) => Caption {
            mark: Mark::Arrow(dir),
            hint: physical_hint(fn_layer, key.label),
        },
        Some(Mark::Text(text)) if text.as_ref() != key.label => Caption {
            mark: Mark::Text(text),
            hint: physical_hint(fn_layer, key.label),
        },
        _ => Caption {
            mark: Mark::Text(Cow::Borrowed(key.label)),
            hint: None,
        },
    }
}

fn physical_hint(fn_layer: bool, label: &'static str) -> Option<&'static str> {
    if fn_layer {
        Some(label)
    } else {
        None
    }
}

fn resolve(record: KeyRecord) -> Option<Mark> {
    if record.is_empty() {
        return None;
    }
    match record.page {
        2 => resolve_keyboard(record),
        13 => {
            let id = keys::function_id(record)?;
            function_name(id).map(|name| Mark::Text(Cow::Borrowed(name)))
        }
        3 => {
            let usage = keys::consumer_usage(record)?;
            media_name(usage).map(|name| Mark::Text(Cow::Borrowed(name)))
        }
        1 => mouse_name(record.b1, record.b2).map(|name| Mark::Text(Cow::Borrowed(name))),
        6 => Some(Mark::Text(Cow::Owned(format!("M{}", record.b1)))),
        8 => Some(Mark::Text(Cow::Owned(format!("DKS{}", record.b1)))),
        9 => Some(Mark::Text(Cow::Borrowed("MT"))),
        10 => Some(Mark::Text(Cow::Borrowed("TGL"))),
        11 => Some(Mark::Text(Cow::Borrowed("SOCD"))),
        12 => Some(Mark::Text(Cow::Borrowed("RS"))),
        _ => None,
    }
}

fn resolve_keyboard(record: KeyRecord) -> Option<Mark> {
    if record.b3 != 0 {
        return None;
    }
    if record.b1 == 0 {
        if let Some(dir) = arrow_dir(record.b2) {
            return Some(Mark::Arrow(dir));
        }
        return hid_text(record.b2).map(|name| Mark::Text(Cow::Borrowed(name)));
    }
    if record.b2 == 0 {
        return modifier_name(record.b1).map(|name| Mark::Text(Cow::Borrowed(name)));
    }
    let modifier = modifier_name(record.b1)?;
    let key = hid_text(record.b2)?;
    Some(Mark::Text(Cow::Owned(format!("{modifier}+{key}"))))
}

fn arrow_dir(hid: u8) -> Option<Dir> {
    match hid {
        0x52 => Some(Dir::Up),
        0x50 => Some(Dir::Left),
        0x51 => Some(Dir::Down),
        0x4F => Some(Dir::Right),
        _ => None,
    }
}

fn hid_text(hid: u8) -> Option<&'static str> {
    if let Some(key) = layout::keys().into_iter().find(|key| key.hid == hid) {
        return Some(key.label);
    }
    Some(match hid {
        0x3A => "F1",
        0x3B => "F2",
        0x3C => "F3",
        0x3D => "F4",
        0x3E => "F5",
        0x3F => "F6",
        0x40 => "F7",
        0x41 => "F8",
        0x42 => "F9",
        0x43 => "F10",
        0x44 => "F11",
        0x45 => "F12",
        0x46 => "Impr Pant",
        0x47 => "Bloq Despl",
        0x48 => "Pausa",
        0x49 => "Insert",
        0x4A => "Inicio",
        0x4B => "Re Pág",
        0x4C => "Supr",
        0x4D => "Fin",
        0x4E => "Av Pág",
        0x35 => "º",
        _ => return None,
    })
}

fn modifier_name(usage: u8) -> Option<&'static str> {
    match usage {
        0xE0 | 0xE4 => Some("Ctrl"),
        0xE1 | 0xE5 => Some("Mayús"),
        0xE2 | 0xE6 => Some("Alt"),
        0xE3 | 0xE7 => Some("Win"),
        _ => None,
    }
}

fn function_name(id: u32) -> Option<&'static str> {
    Some(match id {
        1 => "Restaurar fábrica",
        2 => "BT 1",
        3 => "BT 2",
        4 => "BT 3",
        5 => "Reconexión",
        6 => "Zona F",
        7 => "Batería",
        8 => "Win",
        9 => "Mac",
        10 => "Android",
        11 => "Efecto de luz",
        12 => "Color de luz",
        13 => "Brillo +",
        14 => "Brillo -",
        15 => "Velocidad +",
        16 => "Velocidad -",
        17 => "Apagar luz",
        18 => "Alt+Tab",
        19 => "Win+E",
        20 => "Win+Tab",
        21 => "Win+H",
        22 => "Bloquear Win",
        _ => return None,
    })
}

fn media_name(usage: u16) -> Option<&'static str> {
    Some(match usage {
        233 => "Vol +",
        234 => "Vol -",
        226 => "Silencio",
        205 => "Reproducir",
        183 => "Detener",
        182 => "Anterior",
        181 => "Siguiente",
        _ => return None,
    })
}

fn mouse_name(kind: u8, value: u8) -> Option<&'static str> {
    Some(match (kind, value) {
        (1, 1) => "Clic izq",
        (1, 2) => "Clic der",
        (1, 4) => "Clic med",
        _ => return None,
    })
}
