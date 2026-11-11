use crate::error::{Error, Result};

pub const DIRECTORY: usize = 400;
pub const SLOTS: usize = 100;
pub const FLAG_DOWN: u8 = 0xB0;
pub const FLAG_UP: u8 = 0x30;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub delay_ms: u16,
    pub hid: u8,
    pub down: bool,
}

impl Event {
    pub fn encode(&self) -> [u8; 4] {
        let mut out = [0u8; 4];
        out[0..2].copy_from_slice(&self.delay_ms.to_le_bytes());
        out[2] = self.hid;
        out[3] = if self.down { FLAG_DOWN } else { FLAG_UP };
        out
    }

    pub fn decode(raw: &[u8]) -> Result<Self> {
        if raw.len() < 4 {
            return Err(Error::BadLength {
                what: "evento de macro",
                got: raw.len(),
                expected: 4,
            });
        }
        let down = match raw[3] {
            FLAG_DOWN => true,
            FLAG_UP => false,
            other => {
                return Err(Error::OutOfRange {
                    field: "evento de macro",
                    detail: format!("flag 0x{other:02X}"),
                });
            }
        };
        Ok(Self {
            delay_ms: u16::from_le_bytes([raw[0], raw[1]]),
            hid: raw[2],
            down,
        })
    }
}

pub fn check_delay(delay_ms: u32) -> Result<u16> {
    if delay_ms > u16::MAX as u32 {
        return Err(Error::OutOfRange {
            field: "retardo de macro",
            detail: format!("{delay_ms} ms"),
        });
    }
    Ok(delay_ms as u16)
}

pub fn encode_body(events: &[Event]) -> Result<Vec<u8>> {
    if events.len() > (u16::MAX as usize) / 2 {
        return Err(Error::OutOfRange {
            field: "macro",
            detail: "demasiados eventos".to_string(),
        });
    }
    let mut out = Vec::with_capacity(4 + events.len() * 4);
    let words = (events.len() * 2) as u16;
    out.extend_from_slice(&words.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    for event in events {
        out.extend_from_slice(&event.encode());
    }
    Ok(out)
}

/// Directorio de 100 offsets y los cuerpos pegados a partir del byte 400.
pub fn encode_image(macros: &[Vec<Event>]) -> Result<Vec<u8>> {
    if macros.len() > SLOTS {
        return Err(Error::OutOfRange {
            field: "macros",
            detail: macros.len().to_string(),
        });
    }
    let mut bodies = Vec::new();
    let mut offsets = [0u32; SLOTS];
    let mut cursor = DIRECTORY as u32;
    for (index, events) in macros.iter().enumerate() {
        if events.is_empty() {
            continue;
        }
        offsets[index] = cursor;
        let body = encode_body(events)?;
        let body_len = u32::try_from(body.len()).map_err(|_| Error::OutOfRange {
            field: "macro",
            detail: "el cuerpo no cabe".into(),
        })?;
        let next = cursor
            .checked_add(body_len)
            .ok_or_else(|| Error::OutOfRange {
                field: "imagen de macros",
                detail: "el final no cabe".into(),
            })?;
        if next > u32::from(u16::MAX) {
            return Err(Error::OutOfRange {
                field: "imagen de macros",
                detail: next.to_string(),
            });
        }
        cursor = next;
        bodies.extend_from_slice(&body);
    }
    let mut out = Vec::with_capacity(DIRECTORY + bodies.len());
    for offset in offsets {
        out.extend_from_slice(&offset.to_le_bytes());
    }
    out.extend_from_slice(&bodies);
    Ok(out)
}

/// Inverso de [`encode_image`]. Las ranuras vacías del final no se devuelven.
pub fn decode_image(bytes: &[u8]) -> Result<Vec<Vec<Event>>> {
    if bytes.len() < DIRECTORY {
        return Err(Error::BadLength {
            what: "imagen de macros",
            got: bytes.len(),
            expected: DIRECTORY,
        });
    }
    let mut macros = Vec::new();
    for slot in 0..SLOTS {
        let start = slot * 4;
        let offset_bytes: [u8; 4] = bytes[start..start + 4]
            .try_into()
            .map_err(|_| Error::Protocol("directorio de macros incompleto"))?;
        let offset = u32::from_le_bytes(offset_bytes);
        if offset == 0 {
            macros.push(Vec::new());
            continue;
        }
        let offset = usize::try_from(offset).map_err(|_| Error::OutOfRange {
            field: "offset de macro",
            detail: offset.to_string(),
        })?;
        if offset < DIRECTORY || offset > usize::from(u16::MAX) || offset + 4 > bytes.len() {
            return Err(Error::OutOfRange {
                field: "offset de macro",
                detail: offset.to_string(),
            });
        }
        let words = usize::from(u16::from_le_bytes([bytes[offset], bytes[offset + 1]]));
        let tail = words
            .checked_mul(2)
            .and_then(|body| body.checked_add(4))
            .and_then(|tail| offset.checked_add(tail))
            .ok_or_else(|| Error::OutOfRange {
                field: "imagen de macros",
                detail: "el final no cabe".into(),
            })?;
        if tail > bytes.len() {
            return Err(Error::BadLength {
                what: "imagen de macros",
                got: bytes.len(),
                expected: tail,
            });
        }
        let mut events = Vec::new();
        let mut cursor = offset + 4;
        while cursor + 4 <= tail {
            events.push(Event::decode(&bytes[cursor..cursor + 4])?);
            cursor += 4;
        }
        macros.push(events);
    }
    while macros.last().is_some_and(|events| events.is_empty()) {
        macros.pop();
    }
    Ok(macros)
}
