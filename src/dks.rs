use crate::error::{Error, Result};

pub const SLOT_BYTES: usize = 16;
pub const SLOTS: usize = 64;
pub const TABLE: usize = SLOTS * SLOT_BYTES;
pub const SLOT_LIMIT: u8 = 63;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Slot {
    pub press: u8,
    pub bottom: u8,
    pub release: u8,
    pub reset: u8,
    pub actions: [u8; 4],
    pub masks: [u8; 4],
}

impl Default for Slot {
    fn default() -> Self {
        Self {
            press: 16,
            bottom: 30,
            release: 30,
            reset: 16,
            actions: [0; 4],
            masks: [0; 4],
        }
    }
}

impl Slot {
    pub fn encode(self) -> [u8; SLOT_BYTES] {
        let mut out = [0u8; SLOT_BYTES];
        out[0] = self.press;
        out[1] = self.bottom;
        out[2] = self.release;
        out[3] = self.reset;
        out[5] = self.actions[0];
        out[7] = self.actions[1];
        out[9] = self.actions[2];
        out[11] = self.actions[3];
        out[12..16].copy_from_slice(&self.masks);
        out
    }

    pub fn decode(raw: &[u8]) -> Result<Self> {
        if raw.len() < SLOT_BYTES {
            return Err(Error::BadLength {
                what: "slot DKS",
                got: raw.len(),
                expected: SLOT_BYTES,
            });
        }
        Ok(Self {
            press: raw[0],
            bottom: raw[1],
            release: raw[2],
            reset: raw[3],
            actions: [raw[5], raw[7], raw[9], raw[11]],
            masks: [raw[12], raw[13], raw[14], raw[15]],
        })
    }
}

/// 1.6 mm son 160 centésimas y 16 unidades de 0.1 mm.
pub fn tenths_from_cmm(cmm: u16) -> Result<u8> {
    if !cmm.is_multiple_of(10) {
        return Err(Error::OutOfRange {
            field: "punto DKS",
            detail: format!("{cmm} no cae en una décima"),
        });
    }
    let tenths = cmm / 10;
    if tenths > u8::MAX as u16 {
        return Err(Error::OutOfRange {
            field: "punto DKS",
            detail: tenths.to_string(),
        });
    }
    Ok(tenths as u8)
}

pub fn check_slot(index: u8) -> Result<()> {
    if index >= SLOT_LIMIT {
        Err(Error::OutOfRange {
            field: "slot DKS",
            detail: index.to_string(),
        })
    } else {
        Ok(())
    }
}

pub fn encode_table(slots: &[Slot; SLOTS]) -> [u8; TABLE] {
    let mut out = [0u8; TABLE];
    for (index, slot) in slots.iter().enumerate() {
        let start = index * SLOT_BYTES;
        out[start..start + SLOT_BYTES].copy_from_slice(&slot.encode());
    }
    out
}

pub fn decode_table(bytes: &[u8]) -> Result<[Slot; SLOTS]> {
    if bytes.len() < TABLE {
        return Err(Error::BadLength {
            what: "tabla DKS",
            got: bytes.len(),
            expected: TABLE,
        });
    }
    let mut slots = [Slot::default(); SLOTS];
    for (index, slot) in slots.iter_mut().enumerate() {
        let start = index * SLOT_BYTES;
        *slot = Slot::decode(&bytes[start..start + SLOT_BYTES])?;
    }
    Ok(slots)
}
