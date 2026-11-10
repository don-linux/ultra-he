use crate::error::{Error, Result};

pub const BLOCK: usize = 16;
pub const COLOR_RECORDS: usize = 126;
pub const COLOR_TABLE: usize = COLOR_RECORDS * 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Lighting {
    pub mode: u8,
    pub color: [u8; 3],
    pub color_mode: u8,
    pub brightness: u8,
    pub speed: u8,
    pub direction: u8,
}

impl Default for Lighting {
    fn default() -> Self {
        Self {
            mode: 1,
            color: [0x90, 0xEE, 0x90],
            color_mode: 0,
            brightness: 5,
            speed: 4,
            direction: 0,
        }
    }
}

impl Lighting {
    pub fn encode(self) -> Result<[u8; BLOCK]> {
        if self.brightness > 5 {
            return Err(Error::OutOfRange {
                field: "brillo",
                detail: self.brightness.to_string(),
            });
        }
        if self.speed > 5 {
            return Err(Error::OutOfRange {
                field: "velocidad",
                detail: self.speed.to_string(),
            });
        }
        let mut out = [0u8; BLOCK];
        out[0] = self.mode;
        out[1] = self.color[0];
        out[2] = self.color[1];
        out[3] = self.color[2];
        out[4] = 0xFF;
        out[8] = self.color_mode;
        out[9] = self.brightness;
        out[10] = self.speed;
        out[11] = self.direction;
        out[14] = 0xAA;
        out[15] = 0x55;
        Ok(out)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < BLOCK {
            return Err(Error::BadLength {
                what: "bloque de luz",
                got: bytes.len(),
                expected: BLOCK,
            });
        }
        let mut mode = bytes[0];
        if mode == 21 {
            mode = 0;
        }
        Ok(Self {
            mode,
            color: [bytes[1], bytes[2], bytes[3]],
            color_mode: bytes[8],
            brightness: bytes[9],
            speed: bytes[10],
            direction: bytes[11],
        })
    }
}

pub fn encode_colors(colors: &[[u8; 3]; COLOR_RECORDS]) -> [u8; COLOR_TABLE] {
    let mut out = [0u8; COLOR_TABLE];
    for (index, color) in colors.iter().enumerate() {
        let start = index * 4;
        out[start] = index as u8;
        out[start + 1] = color[0];
        out[start + 2] = color[1];
        out[start + 3] = color[2];
    }
    out
}

pub fn decode_colors(bytes: &[u8]) -> Result<[[u8; 3]; COLOR_RECORDS]> {
    if bytes.len() < COLOR_TABLE {
        return Err(Error::BadLength {
            what: "colores por tecla",
            got: bytes.len(),
            expected: COLOR_TABLE,
        });
    }
    let mut colors = [[0u8; 3]; COLOR_RECORDS];
    for (index, color) in colors.iter_mut().enumerate() {
        let start = index * 4;
        *color = [bytes[start + 1], bytes[start + 2], bytes[start + 3]];
    }
    Ok(colors)
}

pub fn mode_name(mode: u8) -> &'static str {
    match mode {
        0 => "Apagado",
        1 => "Estático",
        2 => "Reactivo",
        3 => "Apagado al pulsar",
        4 => "Cielo estrellado",
        5 => "Nevada",
        6 => "Floración",
        7 => "Respiración",
        8 => "Espectro",
        9 => "Primavera",
        10 => "Cruce",
        11 => "Deriva",
        12 => "Sendero",
        13 => "Onda al pulsar",
        14 => "Doble toque",
        15 => "Ondas",
        16 => "Flujo",
        17 => "Cumbres",
        18 => "Lluvia",
        19 => "Lanzadera",
        20 => "Color por tecla",
        _ => "Modo del firmware",
    }
}
