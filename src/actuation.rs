use crate::error::{Error, Result};
use crate::switches::{self, SwitchChoice, RT_MIN, TRIP_MIN};

pub const RECORDS: usize = 126;
pub const RECORD: usize = 8;
pub const TABLE: usize = RECORDS * RECORD;

pub const FLAG_RT: u8 = 0b0000_0001;
pub const FLAG_RAMPAGE: u8 = 0b0000_0010;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Record {
    pub axis_type: u8,
    pub flags: u8,
    pub trip: u16,
    pub rt_press: u16,
    pub rt_release: u16,
}

impl Default for Record {
    fn default() -> Self {
        Self {
            axis_type: 6,
            flags: 0,
            trip: 120,
            rt_press: 30,
            rt_release: 30,
        }
    }
}

impl Record {
    pub fn encode(self) -> [u8; RECORD] {
        let mut out = [0u8; RECORD];
        out[0] = self.axis_type;
        out[1] = self.flags;
        out[2..4].copy_from_slice(&self.trip.to_le_bytes());
        out[4..6].copy_from_slice(&self.rt_press.to_le_bytes());
        out[6..8].copy_from_slice(&self.rt_release.to_le_bytes());
        out
    }

    pub fn decode(raw: &[u8]) -> Result<Self> {
        if raw.len() < RECORD {
            return Err(Error::BadLength {
                what: "registro de actuación",
                got: raw.len(),
                expected: RECORD,
            });
        }
        Ok(Self {
            axis_type: raw[0],
            flags: raw[1],
            trip: u16::from_le_bytes([raw[2], raw[3]]),
            rt_press: u16::from_le_bytes([raw[4], raw[5]]),
            rt_release: u16::from_le_bytes([raw[6], raw[7]]),
        })
    }

    pub fn rt_enabled(self) -> bool {
        self.flags & FLAG_RT != 0
    }

    pub fn rampage(self) -> bool {
        self.flags & FLAG_RAMPAGE != 0
    }
}

pub fn set_trip(record: &mut Record, choice: SwitchChoice, trip: u16) -> Result<()> {
    switches::check_trip(choice, trip)?;
    record.trip = trip;
    record.axis_type = choice.axis_byte();
    Ok(())
}

pub fn set_rt_press(record: &mut Record, choice: SwitchChoice, value: u16) -> Result<()> {
    switches::check_rt(choice, value)?;
    record.rt_press = value;
    record.flags |= FLAG_RT;
    record.axis_type = choice.axis_byte();
    Ok(())
}

pub fn set_rt_release(record: &mut Record, choice: SwitchChoice, value: u16) -> Result<()> {
    switches::check_rt(choice, value)?;
    record.rt_release = value;
    record.flags |= FLAG_RT;
    record.axis_type = choice.axis_byte();
    Ok(())
}

pub fn set_rt_enabled(record: &mut Record, enabled: bool) {
    if enabled {
        record.flags |= FLAG_RT;
    } else {
        record.flags &= !FLAG_RT;
    }
}

pub fn encode_table(records: &[Record; RECORDS]) -> [u8; TABLE] {
    let mut out = [0u8; TABLE];
    for (index, record) in records.iter().enumerate() {
        out[index * RECORD..(index + 1) * RECORD].copy_from_slice(&record.encode());
    }
    out
}

pub fn decode_table(bytes: &[u8]) -> Result<[Record; RECORDS]> {
    if bytes.len() < TABLE {
        return Err(Error::BadLength {
            what: "tabla de actuación",
            got: bytes.len(),
            expected: TABLE,
        });
    }
    let mut records = [Record::default(); RECORDS];
    for (index, record) in records.iter_mut().enumerate() {
        let start = index * RECORD;
        *record = Record::decode(&bytes[start..start + RECORD])?;
    }
    Ok(records)
}

pub fn cmm_from_mm_text(text: &str) -> Result<u16> {
    let text = text.trim();
    let negative = text.starts_with('-');
    if negative {
        return Err(Error::OutOfRange {
            field: "milímetros",
            detail: text.to_string(),
        });
    }
    let (whole, frac) = text.split_once('.').unwrap_or((text, ""));
    if frac.len() > 2 || whole.is_empty() && frac.is_empty() {
        return Err(Error::Json(format!("milímetros inválidos: {text}")));
    }
    let whole: u16 = if whole.is_empty() {
        0
    } else {
        whole
            .parse()
            .map_err(|_| Error::Json(format!("milímetros inválidos: {text}")))?
    };
    let mut frac_buf = frac.to_string();
    while frac_buf.len() < 2 {
        frac_buf.push('0');
    }
    let frac: u16 = frac_buf
        .parse()
        .map_err(|_| Error::Json(format!("milímetros inválidos: {text}")))?;
    Ok(whole * 100 + frac)
}

pub fn mm_text(cmm: u16) -> String {
    format!("{}.{:02}", cmm / 100, cmm % 100)
}

pub fn validate_record(choice: SwitchChoice, record: &Record) -> Result<()> {
    if record.axis_type != choice.axis_byte() {
        return Err(Error::Protocol("el byte de eje no coincide con el perfil"));
    }
    if record.trip < TRIP_MIN {
        return Err(Error::OutOfRange {
            field: "actuación",
            detail: record.trip.to_string(),
        });
    }
    switches::check_trip(choice, record.trip)?;
    if record.rt_press < RT_MIN || record.rt_release < RT_MIN {
        return Err(Error::OutOfRange {
            field: "rapid trigger",
            detail: format!(
                "presión {} liberación {}",
                record.rt_press, record.rt_release
            ),
        });
    }
    switches::check_rt(choice, record.rt_press)?;
    switches::check_rt(choice, record.rt_release)?;
    Ok(())
}
