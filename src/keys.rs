use crate::error::{Error, Result};

pub const RECORD: usize = 4;
pub const SLOTS: usize = 128;
pub const BASE_READ: usize = SLOTS * RECORD;
pub const BASE_WRITE: usize = 126 * RECORD;
pub const FN_BYTES: usize = SLOTS * RECORD;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct KeyRecord {
    pub page: u8,
    pub b1: u8,
    pub b2: u8,
    pub b3: u8,
}

impl KeyRecord {
    pub fn encode(self) -> [u8; RECORD] {
        [self.page, self.b1, self.b2, self.b3]
    }

    pub fn decode(raw: &[u8]) -> Result<Self> {
        if raw.len() < RECORD {
            return Err(Error::BadLength {
                what: "registro de tecla",
                got: raw.len(),
                expected: RECORD,
            });
        }
        Ok(Self {
            page: raw[0],
            b1: raw[1],
            b2: raw[2],
            b3: raw[3],
        })
    }

    pub fn is_empty(self) -> bool {
        self.page == 0 && self.b1 == 0 && self.b2 == 0 && self.b3 == 0
    }
}

pub fn keyboard(modifier: u8, hid: u8) -> KeyRecord {
    KeyRecord {
        page: 2,
        b1: modifier,
        b2: hid,
        b3: 0,
    }
}

pub fn mouse(button: u8, value: u8) -> KeyRecord {
    KeyRecord {
        page: 1,
        b1: button,
        b2: value,
        b3: 0,
    }
}

pub fn consumer(usage: u16) -> KeyRecord {
    let bytes = usage.to_le_bytes();
    KeyRecord {
        page: 3,
        b1: bytes[0],
        b2: bytes[1],
        b3: 0,
    }
}

pub fn consumer_usage(record: KeyRecord) -> Option<u16> {
    if record.page == 3 {
        Some(u16::from_le_bytes([record.b1, record.b2]))
    } else {
        None
    }
}

pub fn macro_bind(slot: u8, mode: u8, count: u8) -> KeyRecord {
    KeyRecord {
        page: 6,
        b1: slot,
        b2: mode,
        b3: count,
    }
}

pub fn dks_bind(slot: u8) -> Result<KeyRecord> {
    if slot >= 63 {
        return Err(Error::OutOfRange {
            field: "slot DKS",
            detail: slot.to_string(),
        });
    }
    Ok(KeyRecord {
        page: 8,
        b1: slot,
        b2: 0,
        b3: 0,
    })
}

pub fn mod_tap(tap: u8, hold: u8, delay_byte: u8) -> KeyRecord {
    KeyRecord {
        page: 9,
        b1: tap,
        b2: hold,
        b3: delay_byte,
    }
}

pub fn toggle(hid: u8) -> KeyRecord {
    KeyRecord {
        page: 10,
        b1: hid,
        b2: 0,
        b3: 0,
    }
}

pub fn socd(mode: u8, hid_a: u8, hid_b: u8) -> KeyRecord {
    KeyRecord {
        page: 11,
        b1: mode,
        b2: hid_a,
        b3: hid_b,
    }
}

pub fn rapid_snap(hid_a: u8, hid_b: u8) -> KeyRecord {
    KeyRecord {
        page: 12,
        b1: 0,
        b2: hid_a,
        b3: hid_b,
    }
}

pub fn device_function(id: u32) -> KeyRecord {
    KeyRecord {
        page: 13,
        b1: ((id >> 16) & 0xFF) as u8,
        b2: ((id >> 8) & 0xFF) as u8,
        b3: (id & 0xFF) as u8,
    }
}

pub fn function_id(record: KeyRecord) -> Option<u32> {
    if record.page == 13 {
        Some(((record.b1 as u32) << 16) | ((record.b2 as u32) << 8) | record.b3 as u32)
    } else {
        None
    }
}

pub fn encode_table(records: &[KeyRecord; SLOTS]) -> [u8; BASE_READ] {
    let mut out = [0u8; BASE_READ];
    for (index, record) in records.iter().enumerate() {
        out[index * RECORD..(index + 1) * RECORD].copy_from_slice(&record.encode());
    }
    out
}

pub fn decode_table(bytes: &[u8]) -> Result<[KeyRecord; SLOTS]> {
    if bytes.len() < BASE_READ {
        return Err(Error::BadLength {
            what: "tabla de teclas",
            got: bytes.len(),
            expected: BASE_READ,
        });
    }
    let mut records = [KeyRecord::default(); SLOTS];
    for (index, record) in records.iter_mut().enumerate() {
        let start = index * RECORD;
        *record = KeyRecord::decode(&bytes[start..start + RECORD])?;
    }
    Ok(records)
}

/// El mapa base se escribe sin los dos últimos registros.
pub fn base_write_bytes(records: &[KeyRecord; SLOTS]) -> Result<Vec<u8>> {
    let full = encode_table(records);
    if full[BASE_WRITE..].iter().any(|byte| *byte != 0) {
        return Err(Error::Protocol(
            "los registros 126 y 127 del mapa base no están en cero",
        ));
    }
    Ok(full[..BASE_WRITE].to_vec())
}

pub fn fn_write_bytes(records: &[KeyRecord; SLOTS]) -> Vec<u8> {
    encode_table(records).to_vec()
}

pub fn apply_pair(
    records: &mut [KeyRecord; SLOTS],
    a: usize,
    b: usize,
    record: KeyRecord,
) -> Result<()> {
    if a == b || a >= 126 || b >= 126 {
        return Err(Error::Protocol(
            "el par necesita dos índices distintos entre 0 y 125",
        ));
    }
    records[a] = record;
    records[b] = record;
    Ok(())
}

pub fn clear_pair(records: &mut [KeyRecord; SLOTS], a: usize, b: usize) -> Result<()> {
    apply_pair(records, a, b, KeyRecord::default())
}
