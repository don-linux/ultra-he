use crate::keys::{self, KeyRecord};
use crate::layout;

/// Acción de una tecla, con la misma codificación que hec.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Binding {
    Empty,
    Key {
        hid: u8,
    },
    Modifier {
        usage: u8,
    },
    Media {
        usage: u16,
    },
    Mouse {
        kind: u8,
        value: u8,
    },
    Function {
        id: u32,
    },
    Macro {
        slot: u8,
        mode: u8,
        count: u8,
    },
    Dks {
        slot: u8,
    },
    ModTap {
        tap: u8,
        hold: u8,
        delay: u8,
    },
    Toggle {
        hid: u8,
    },
    Socd {
        mode: u8,
        hid_a: u8,
        hid_b: u8,
    },
    RapidSnap {
        hid_a: u8,
        hid_b: u8,
    },
    /// Bytes que no entran en la tabla de hec. Se conservan al escribir.
    Custom(KeyRecord),
}

impl Binding {
    pub fn record(self) -> KeyRecord {
        match self {
            Binding::Empty => KeyRecord::default(),
            Binding::Key { hid } => keys::keyboard(0, hid),
            Binding::Modifier { usage } => keys::keyboard(usage, 0),
            Binding::Media { usage } => keys::consumer(usage),
            Binding::Mouse { kind, value } => keys::mouse(kind, value),
            Binding::Function { id } => keys::device_function(id),
            Binding::Macro { slot, mode, count } => keys::macro_bind(slot, mode, count),
            Binding::Dks { slot } => KeyRecord {
                page: 8,
                b1: slot,
                b2: 0,
                b3: 0,
            },
            Binding::ModTap { tap, hold, delay } => keys::mod_tap(tap, hold, delay),
            Binding::Toggle { hid } => keys::toggle(hid),
            Binding::Socd { mode, hid_a, hid_b } => keys::socd(mode, hid_a, hid_b),
            Binding::RapidSnap { hid_a, hid_b } => keys::rapid_snap(hid_a, hid_b),
            Binding::Custom(record) => record,
        }
    }

    pub fn label(self) -> String {
        match self {
            Binding::Empty => "Vacío".into(),
            Binding::Key { hid } => hid_label(hid).to_string(),
            Binding::Modifier { usage } => hid_label(usage).to_string(),
            Binding::Media { usage } => media_label(usage).to_string(),
            Binding::Mouse { kind, value } => mouse_label(kind, value).to_string(),
            Binding::Function { id } => function_label(id).to_string(),
            Binding::Macro { slot, .. } => format!("Macro {slot}"),
            Binding::Dks { slot } => format!("DKS {slot}"),
            Binding::ModTap { .. } => "Mod-tap".into(),
            Binding::Toggle { .. } => "Toggle".into(),
            Binding::Socd { .. } => "SOCD".into(),
            Binding::RapidSnap { .. } => "Rapid snap".into(),
            Binding::Custom(_) => "Personalizada".into(),
        }
    }
}

pub fn decode(record: KeyRecord) -> Binding {
    if record.is_empty() {
        return Binding::Empty;
    }
    match record.page {
        2 if (0xE0..=0xE7).contains(&record.b1) && record.b2 == 0 && record.b3 == 0 => {
            Binding::Modifier { usage: record.b1 }
        }
        2 if record.b1 == 0 && record.b3 == 0 => Binding::Key { hid: record.b2 },
        3 if record.b3 == 0 => Binding::Media {
            usage: u16::from_le_bytes([record.b1, record.b2]),
        },
        1 => Binding::Mouse {
            kind: record.b1,
            value: record.b2,
        },
        13 => Binding::Function {
            id: keys::function_id(record).unwrap_or(0),
        },
        6 => Binding::Macro {
            slot: record.b1,
            mode: record.b2,
            count: record.b3,
        },
        8 if record.b2 == 0 && record.b3 == 0 => Binding::Dks { slot: record.b1 },
        9 => Binding::ModTap {
            tap: record.b1,
            hold: record.b2,
            delay: record.b3,
        },
        10 if record.b2 == 0 && record.b3 == 0 => Binding::Toggle { hid: record.b1 },
        11 => Binding::Socd {
            mode: record.b1,
            hid_a: record.b2,
            hid_b: record.b3,
        },
        12 => Binding::RapidSnap {
            hid_a: record.b2,
            hid_b: record.b3,
        },
        _ => Binding::Custom(record),
    }
}

pub fn label_of(record: KeyRecord) -> String {
    decode(record).label()
}

/// Nombre de archivo SVG de la leyenda, si la acción es una tecla con icono.
pub fn svg_name(record: KeyRecord) -> Option<&'static str> {
    let hid = match decode(record) {
        Binding::Key { hid } | Binding::Modifier { usage: hid } => hid,
        _ => return None,
    };
    layout::keys()
        .into_iter()
        .find(|key| key.hid == hid)
        .and_then(|key| key.svg)
}

#[derive(Debug, Clone, Copy)]
pub struct FactorySlot {
    pub index: u16,
    pub label: &'static str,
    pub record: KeyRecord,
    pub locked: bool,
}

pub fn factory_at(index: u16) -> Option<FactorySlot> {
    FACTORY.iter().copied().find(|slot| slot.index == index)
}

pub fn fn_locked(index: u16) -> bool {
    factory_at(index).is_some_and(|slot| slot.locked)
}

const fn key(hid: u8) -> KeyRecord {
    KeyRecord {
        page: 2,
        b1: 0,
        b2: hid,
        b3: 0,
    }
}

const fn fun(id: u8) -> KeyRecord {
    KeyRecord {
        page: 13,
        b1: 0,
        b2: 0,
        b3: id,
    }
}

/// Mapa Fn de fábrica de hec (`get2860`). F1..F12 van fijas.
const FACTORY: &[FactorySlot] = &[
    FactorySlot {
        index: 0,
        label: "Restaurar fábrica",
        record: fun(1),
        locked: false,
    },
    FactorySlot {
        index: 17,
        label: "F1",
        record: key(0x3A),
        locked: true,
    },
    FactorySlot {
        index: 18,
        label: "F2",
        record: key(0x3B),
        locked: true,
    },
    FactorySlot {
        index: 19,
        label: "F3",
        record: key(0x3C),
        locked: true,
    },
    FactorySlot {
        index: 20,
        label: "F4",
        record: key(0x3D),
        locked: true,
    },
    FactorySlot {
        index: 21,
        label: "F5",
        record: key(0x3E),
        locked: true,
    },
    FactorySlot {
        index: 22,
        label: "F6",
        record: key(0x3F),
        locked: true,
    },
    FactorySlot {
        index: 23,
        label: "F7",
        record: key(0x40),
        locked: true,
    },
    FactorySlot {
        index: 24,
        label: "F8",
        record: key(0x41),
        locked: true,
    },
    FactorySlot {
        index: 25,
        label: "F9",
        record: key(0x42),
        locked: true,
    },
    FactorySlot {
        index: 26,
        label: "F10",
        record: key(0x43),
        locked: true,
    },
    FactorySlot {
        index: 27,
        label: "F11",
        record: key(0x44),
        locked: true,
    },
    FactorySlot {
        index: 28,
        label: "F12",
        record: key(0x45),
        locked: true,
    },
    FactorySlot {
        index: 92,
        label: "Apagar luz",
        record: fun(17),
        locked: false,
    },
    FactorySlot {
        index: 32,
        label: "º",
        record: key(0x35),
        locked: false,
    },
    FactorySlot {
        index: 33,
        label: "BT 1",
        record: fun(2),
        locked: false,
    },
    FactorySlot {
        index: 34,
        label: "BT 2",
        record: fun(3),
        locked: false,
    },
    FactorySlot {
        index: 35,
        label: "BT 3",
        record: fun(4),
        locked: false,
    },
    FactorySlot {
        index: 36,
        label: "Reconexión",
        record: fun(5),
        locked: false,
    },
    FactorySlot {
        index: 39,
        label: "Impr Pant",
        record: key(0x46),
        locked: false,
    },
    FactorySlot {
        index: 40,
        label: "Bloq Despl",
        record: key(0x47),
        locked: false,
    },
    FactorySlot {
        index: 41,
        label: "Pausa",
        record: key(0x48),
        locked: false,
    },
    FactorySlot {
        index: 60,
        label: "Efecto de luz",
        record: fun(11),
        locked: false,
    },
    FactorySlot {
        index: 49,
        label: "Android",
        record: fun(10),
        locked: false,
    },
    FactorySlot {
        index: 50,
        label: "Win",
        record: fun(8),
        locked: false,
    },
    FactorySlot {
        index: 51,
        label: "Mac",
        record: fun(9),
        locked: false,
    },
    FactorySlot {
        index: 56,
        label: "Inicio",
        record: key(0x4A),
        locked: false,
    },
    FactorySlot {
        index: 57,
        label: "Fin",
        record: key(0x4D),
        locked: false,
    },
    FactorySlot {
        index: 58,
        label: "Re Pág",
        record: key(0x4B),
        locked: false,
    },
    FactorySlot {
        index: 59,
        label: "Av Pág",
        record: key(0x4E),
        locked: false,
    },
    FactorySlot {
        index: 76,
        label: "Color de luz",
        record: fun(12),
        locked: false,
    },
    FactorySlot {
        index: 69,
        label: "Batería",
        record: fun(7),
        locked: false,
    },
    FactorySlot {
        index: 72,
        label: "Velocidad -",
        record: fun(16),
        locked: false,
    },
    FactorySlot {
        index: 73,
        label: "Velocidad +",
        record: fun(15),
        locked: false,
    },
    FactorySlot {
        index: 74,
        label: "Insert",
        record: key(0x49),
        locked: false,
    },
    FactorySlot {
        index: 75,
        label: "Supr",
        record: key(0x4C),
        locked: false,
    },
    FactorySlot {
        index: 80,
        label: "Zona F",
        record: fun(6),
        locked: false,
    },
    FactorySlot {
        index: 81,
        label: "Bloquear Win",
        record: fun(22),
        locked: false,
    },
];

#[derive(Debug, Clone, Copy)]
pub struct PaletteItem {
    pub group: &'static str,
    pub label: &'static str,
    pub record: KeyRecord,
}

pub fn palette() -> Vec<PaletteItem> {
    let mut items = Vec::new();
    for (hid, label) in BASIC {
        items.push(PaletteItem {
            group: "Básicas",
            label,
            record: key(*hid),
        });
    }
    for (usage, label) in MODIFIERS {
        items.push(PaletteItem {
            group: "Modificadores",
            label,
            record: KeyRecord {
                page: 2,
                b1: *usage,
                b2: 0,
                b3: 0,
            },
        });
    }
    for (hid, label) in NAV {
        items.push(PaletteItem {
            group: "Navegación/F",
            label,
            record: key(*hid),
        });
    }
    for (hid, label) in NUMPAD {
        items.push(PaletteItem {
            group: "Numpad",
            label,
            record: key(*hid),
        });
    }
    for (usage, label) in MEDIA {
        items.push(PaletteItem {
            group: "Multimedia",
            label,
            record: keys::consumer(*usage),
        });
    }
    for (kind, value, label) in MOUSE {
        items.push(PaletteItem {
            group: "Ratón",
            label,
            record: keys::mouse(*kind, *value),
        });
    }
    for id in 2u8..=22 {
        items.push(PaletteItem {
            group: "Funciones",
            label: function_label(id as u32),
            record: fun(id),
        });
    }
    items
}

const BASIC: &[(u8, &str)] = &[
    (0x29, "Esc"),
    (0x1E, "1"),
    (0x1F, "2"),
    (0x20, "3"),
    (0x21, "4"),
    (0x22, "5"),
    (0x23, "6"),
    (0x24, "7"),
    (0x25, "8"),
    (0x26, "9"),
    (0x27, "0"),
    (0x2D, "-"),
    (0x2E, "="),
    (0x2A, "Retroceso"),
    (0x2B, "Tab"),
    (0x14, "Q"),
    (0x1A, "W"),
    (0x08, "E"),
    (0x15, "R"),
    (0x17, "T"),
    (0x1C, "Y"),
    (0x18, "U"),
    (0x0C, "I"),
    (0x12, "O"),
    (0x13, "P"),
    (0x2F, "["),
    (0x30, "]"),
    (0x31, "\\"),
    (0x39, "Bloq Mayús"),
    (0x04, "A"),
    (0x16, "S"),
    (0x07, "D"),
    (0x09, "F"),
    (0x0A, "G"),
    (0x0B, "H"),
    (0x0D, "J"),
    (0x0E, "K"),
    (0x0F, "L"),
    (0x33, ";"),
    (0x34, "'"),
    (0x28, "Enter"),
    (0x1D, "Z"),
    (0x1B, "X"),
    (0x06, "C"),
    (0x19, "V"),
    (0x05, "B"),
    (0x11, "N"),
    (0x10, "M"),
    (0x36, ","),
    (0x37, "."),
    (0x38, "/"),
    (0x2C, "Espacio"),
    (0x35, "º"),
    (0x65, "Menú"),
];

const MODIFIERS: &[(u8, &str)] = &[
    (0xE0, "Ctrl"),
    (0xE1, "Mayús"),
    (0xE2, "Alt"),
    (0xE3, "Win"),
    (0xE4, "Ctrl derecho"),
    (0xE5, "Mayús derecho"),
    (0xE6, "Alt Gr"),
    (0xE7, "Win derecho"),
];

const NAV: &[(u8, &str)] = &[
    (0x52, "↑"),
    (0x51, "↓"),
    (0x4F, "→"),
    (0x50, "←"),
    (0x4A, "Inicio"),
    (0x4D, "Fin"),
    (0x49, "Insert"),
    (0x4C, "Supr"),
    (0x4B, "Re Pág"),
    (0x4E, "Av Pág"),
    (0x46, "Impr Pant"),
    (0x47, "Bloq Despl"),
    (0x48, "Pausa"),
    (0x3A, "F1"),
    (0x3B, "F2"),
    (0x3C, "F3"),
    (0x3D, "F4"),
    (0x3E, "F5"),
    (0x3F, "F6"),
    (0x40, "F7"),
    (0x41, "F8"),
    (0x42, "F9"),
    (0x43, "F10"),
    (0x44, "F11"),
    (0x45, "F12"),
];

const NUMPAD: &[(u8, &str)] = &[
    (0x53, "Bloq Num"),
    (0x54, "Num /"),
    (0x55, "Num *"),
    (0x56, "Num -"),
    (0x57, "Num +"),
    (0x58, "Num Enter"),
    (0x59, "Num 1"),
    (0x5A, "Num 2"),
    (0x5B, "Num 3"),
    (0x5C, "Num 4"),
    (0x5D, "Num 5"),
    (0x5E, "Num 6"),
    (0x5F, "Num 7"),
    (0x60, "Num 8"),
    (0x61, "Num 9"),
    (0x62, "Num 0"),
    (0x63, "Num ."),
];

const MEDIA: &[(u16, &str)] = &[
    (404, "Mi PC"),
    (402, "Calculadora"),
    (394, "Correo"),
    (387, "Multimedia"),
    (205, "Reproducir"),
    (183, "Detener"),
    (182, "Anterior"),
    (181, "Siguiente"),
    (233, "Vol +"),
    (234, "Vol -"),
    (226, "Silencio"),
    (547, "Inicio navegador"),
    (551, "Actualizar"),
    (549, "Adelante"),
    (548, "Atrás"),
    (554, "Favoritos"),
    (545, "Buscar"),
];

const MOUSE: &[(u8, u8, &str)] = &[
    (1, 1, "Clic izquierdo"),
    (1, 4, "Clic central"),
    (1, 2, "Clic derecho"),
    (3, 1, "Rueda arriba"),
    (3, 255, "Rueda abajo"),
    (1, 16, "Adelante"),
    (1, 8, "Atrás"),
];

pub fn function_label(id: u32) -> &'static str {
    match id {
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
        _ => "Función",
    }
}

fn hid_label(hid: u8) -> &'static str {
    BASIC
        .iter()
        .chain(NAV)
        .chain(NUMPAD)
        .find(|(code, _)| *code == hid)
        .map(|(_, label)| *label)
        .or_else(|| {
            MODIFIERS
                .iter()
                .find(|(code, _)| *code == hid)
                .map(|(_, label)| *label)
        })
        .unwrap_or("Tecla")
}

fn media_label(usage: u16) -> &'static str {
    MEDIA
        .iter()
        .find(|(code, _)| *code == usage)
        .map(|(_, label)| *label)
        .unwrap_or("Multimedia")
}

fn mouse_label(kind: u8, value: u8) -> &'static str {
    MOUSE
        .iter()
        .find(|(k, v, _)| *k == kind && *v == value)
        .map(|(_, _, label)| *label)
        .unwrap_or("Ratón")
}

/// Traduce una tecla de egui al uso HID de teclado. `None` si no hay equivalente.
pub fn hid_from_egui_name(name: &str) -> Option<u8> {
    Some(match name {
        "Escape" => 0x29,
        "Tab" => 0x2B,
        "Enter" => 0x28,
        "Space" => 0x2C,
        "Backspace" => 0x2A,
        "A" => 0x04,
        "B" => 0x05,
        "C" => 0x06,
        "D" => 0x07,
        "E" => 0x08,
        "F" => 0x09,
        "G" => 0x0A,
        "H" => 0x0B,
        "I" => 0x0C,
        "J" => 0x0D,
        "K" => 0x0E,
        "L" => 0x0F,
        "M" => 0x10,
        "N" => 0x11,
        "O" => 0x12,
        "P" => 0x13,
        "Q" => 0x14,
        "R" => 0x15,
        "S" => 0x16,
        "T" => 0x17,
        "U" => 0x18,
        "V" => 0x19,
        "W" => 0x1A,
        "X" => 0x1B,
        "Y" => 0x1C,
        "Z" => 0x1D,
        "0" | "Num0" => 0x27,
        "1" => 0x1E,
        "2" => 0x1F,
        "3" => 0x20,
        "4" => 0x21,
        "5" => 0x22,
        "6" => 0x23,
        "7" => 0x24,
        "8" => 0x25,
        "9" => 0x26,
        "F1" => 0x3A,
        "F2" => 0x3B,
        "F3" => 0x3C,
        "F4" => 0x3D,
        "F5" => 0x3E,
        "F6" => 0x3F,
        "F7" => 0x40,
        "F8" => 0x41,
        "F9" => 0x42,
        "F10" => 0x43,
        "F11" => 0x44,
        "F12" => 0x45,
        "ArrowUp" => 0x52,
        "ArrowDown" => 0x51,
        "ArrowRight" => 0x4F,
        "ArrowLeft" => 0x50,
        _ => return None,
    })
}
