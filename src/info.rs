use crate::error::{Error, Result};

pub const TABLE: usize = 56;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    pub vid: u16,
    pub pid: u16,
    pub firmware: String,
    pub battery: u8,
}

pub fn parse(payload: &[u8]) -> Result<DeviceInfo> {
    if payload.len() < 18 {
        return Err(Error::BadLength {
            what: "información del dispositivo",
            got: payload.len(),
            expected: TABLE,
        });
    }
    let vid = u16::from_le_bytes([payload[4], payload[5]]);
    let pid = u16::from_le_bytes([payload[6], payload[7]]);
    let major = payload[8];
    let minor = payload[9];
    let hundredths = (major & 0x0F) as u16 + ((major >> 4) as u16) * 10 + minor as u16 * 100;
    Ok(DeviceInfo {
        vid,
        pid,
        firmware: format!("{}.{:02}", hundredths / 100, hundredths % 100),
        battery: payload[17],
    })
}
