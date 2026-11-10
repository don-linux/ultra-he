use crate::error::{Error, Result};

pub const TABLE: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Globals {
    pub raw: [u8; TABLE],
}

impl Default for Globals {
    fn default() -> Self {
        let mut raw = [0u8; TABLE];
        raw[3] = 1;
        raw[5] = 6;
        raw[11] = 1;
        raw[14] = 1;
        Self { raw }
    }
}

impl Globals {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < TABLE {
            return Err(Error::BadLength {
                what: "tabla global",
                got: bytes.len(),
                expected: TABLE,
            });
        }
        let mut raw = [0u8; TABLE];
        raw.copy_from_slice(&bytes[..TABLE]);
        Ok(Self { raw })
    }

    pub fn report_rate(&self) -> u8 {
        self.raw[5]
    }

    pub fn set_report_rate(&mut self, rate: u8) -> Result<()> {
        if rate > 6 {
            return Err(Error::OutOfRange {
                field: "tasa",
                detail: format!("{rate}, permitido 0..=6"),
            });
        }
        self.raw[5] = rate;
        Ok(())
    }

    pub fn sleep_minutes(&self) -> u8 {
        self.raw[3]
    }

    pub fn set_sleep_minutes(&mut self, minutes: u8) -> Result<()> {
        if minutes > 30 {
            return Err(Error::OutOfRange {
                field: "suspensión",
                detail: format!("{minutes}, permitido 0..=30"),
            });
        }
        self.raw[3] = minutes;
        Ok(())
    }

    pub fn top_dead_cmm(&self) -> u8 {
        self.raw[8]
    }

    pub fn bottom_dead_cmm(&self) -> u8 {
        self.raw[9]
    }

    pub fn set_top_dead(&mut self, cmm: u8) -> Result<()> {
        set_dead(&mut self.raw[8], cmm)
    }

    pub fn set_bottom_dead(&mut self, cmm: u8) -> Result<()> {
        set_dead(&mut self.raw[9], cmm)
    }

    pub fn stability(&self) -> bool {
        self.raw[11] != 0
    }

    pub fn set_stability(&mut self, on: bool) {
        self.raw[11] = u8::from(on);
    }

    pub fn auto_calibration(&self) -> bool {
        self.raw[14] != 0
    }

    pub fn set_auto_calibration(&mut self, on: bool) {
        self.raw[14] = u8::from(on);
    }
}

fn set_dead(slot: &mut u8, cmm: u8) -> Result<()> {
    if cmm > 50 {
        return Err(Error::OutOfRange {
            field: "zona muerta",
            detail: format!("{cmm} centésimas, máximo 50"),
        });
    }
    *slot = cmm;
    Ok(())
}
