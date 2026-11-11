use crate::error::{Error, Result};
use crate::switches::{SwitchChoice, RT_MIN, TRIP_MIN};

pub const RECORDS: usize = 126;
pub const RECORD: usize = 8;
pub const TABLE: usize = RECORDS * RECORD;

/// Bit 0 de hec: "Activación rápida de todo el recorrido".
pub const FLAG_FULL_TRAVEL: u8 = 0b0000_0001;
/// Bit 1 de hec: "Optimización de fondo". Este teclado no la muestra.
pub const FLAG_BOTTOM_OPT: u8 = 0b0000_0010;
/// Bit 2 de hec: Rampage. El MINI 60 HE PRO tiene `isModel: false`.
pub const FLAG_RAMPAGE: u8 = 0b0000_0100;

/// Sensibilidad que hec escribe al encender Rapid Trigger (`sensitivity: 10`).
pub const RT_DEFAULT: u16 = 10;
/// Recorrido que hec muestra cuando el firmware trae 0 (`currentTrip: 12`).
pub const TRIP_DEFAULT: u16 = 120;

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

    /// hec: Rapid Trigger está activo solo si presión y liberación son distintas de 0.
    pub fn rt_enabled(self) -> bool {
        self.rt_press != 0 && self.rt_release != 0
    }

    pub fn full_travel(self) -> bool {
        self.flags & FLAG_FULL_TRAVEL != 0
    }

    pub fn bottom_opt(self) -> bool {
        self.flags & FLAG_BOTTOM_OPT != 0
    }

    pub fn rampage(self) -> bool {
        self.flags & FLAG_RAMPAGE != 0
    }

    /// Valor que se muestra. Un 0 leído se enseña como 1.20 mm y se escribe tal cual.
    pub fn display_trip(self) -> u16 {
        if self.trip == 0 {
            TRIP_DEFAULT
        } else {
            self.trip
        }
    }
}

pub fn set_trip(record: &mut Record, choice: SwitchChoice, trip: u16) -> Result<()> {
    let max = choice.max_cmm()?;
    set_trip_capped(record, choice, trip, max)
}

pub fn set_trip_capped(
    record: &mut Record,
    choice: SwitchChoice,
    trip: u16,
    max: u16,
) -> Result<()> {
    if trip < TRIP_MIN || trip > max {
        return Err(Error::OutOfRange {
            field: "actuación",
            detail: format!("{trip} centésimas, permitido {TRIP_MIN}..={max}"),
        });
    }
    record.trip = trip;
    record.axis_type = choice.axis_byte();
    Ok(())
}

pub fn set_rt_press(record: &mut Record, choice: SwitchChoice, value: u16) -> Result<()> {
    let max = choice.max_cmm()?;
    set_rt_press_capped(record, choice, value, max)
}

pub fn set_rt_press_capped(
    record: &mut Record,
    choice: SwitchChoice,
    value: u16,
    max: u16,
) -> Result<()> {
    check_edited_rt(value, max)?;
    record.rt_press = value;
    record.axis_type = choice.axis_byte();
    Ok(())
}

pub fn set_rt_release(record: &mut Record, choice: SwitchChoice, value: u16) -> Result<()> {
    let max = choice.max_cmm()?;
    set_rt_release_capped(record, choice, value, max)
}

pub fn set_rt_release_capped(
    record: &mut Record,
    choice: SwitchChoice,
    value: u16,
    max: u16,
) -> Result<()> {
    check_edited_rt(value, max)?;
    record.rt_release = value;
    record.axis_type = choice.axis_byte();
    Ok(())
}

fn check_edited_rt(value: u16, max: u16) -> Result<()> {
    if value < RT_MIN || value > max {
        return Err(Error::OutOfRange {
            field: "rapid trigger",
            detail: format!("{value} centésimas, permitido {RT_MIN}..={max}"),
        });
    }
    Ok(())
}

/// Encender usa 0.10 mm donde haya un 0. Apagar escribe 0/0 y limpia el bit 0.
/// Los bits 1 y 2 se conservan: no hay un control que los restaure.
pub fn set_rt_enabled(record: &mut Record, enabled: bool) {
    if enabled {
        if record.rt_press == 0 {
            record.rt_press = RT_DEFAULT;
        }
        if record.rt_release == 0 {
            record.rt_release = RT_DEFAULT;
        }
    } else {
        record.rt_press = 0;
        record.rt_release = 0;
        record.flags &= !FLAG_FULL_TRAVEL;
    }
}

pub fn set_full_travel(record: &mut Record, enabled: bool) {
    if enabled {
        record.flags |= FLAG_FULL_TRAVEL;
    } else {
        record.flags &= !FLAG_FULL_TRAVEL;
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
    let whole: u32 = if whole.is_empty() {
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
    let frac: u32 = frac_buf
        .parse()
        .map_err(|_| Error::Json(format!("milímetros inválidos: {text}")))?;
    let total = whole
        .checked_mul(100)
        .and_then(|value| value.checked_add(frac))
        .ok_or_else(|| Error::OutOfRange {
            field: "milímetros",
            detail: text.to_string(),
        })?;
    u16::try_from(total).map_err(|_| Error::OutOfRange {
        field: "milímetros",
        detail: text.to_string(),
    })
}

pub fn mm_text(cmm: u16) -> String {
    format!("{}.{:02}", cmm / 100, cmm % 100)
}

/// Acepta recorrido 0 y Rapid Trigger 0/0, que es lo que devuelve el teclado.
/// Un valor editado distinto de 0 sigue teniendo mínimo y tope.
pub fn validate_record(choice: SwitchChoice, record: &Record, max: u16) -> Result<()> {
    if record.axis_type != choice.axis_byte() {
        return Err(Error::Protocol("el byte de eje no coincide con el perfil"));
    }
    if record.trip != 0 {
        check_span("actuación", record.trip, TRIP_MIN, max)?;
    }
    if record.rt_press != 0 {
        check_span("rapid trigger", record.rt_press, RT_MIN, max)?;
    }
    if record.rt_release != 0 {
        check_span("rapid trigger", record.rt_release, RT_MIN, max)?;
    }
    Ok(())
}

fn check_span(field: &'static str, value: u16, min: u16, max: u16) -> Result<()> {
    if value < min || value > max {
        return Err(Error::OutOfRange {
            field,
            detail: format!("{value} centésimas, permitido {min}..={max}"),
        });
    }
    Ok(())
}
