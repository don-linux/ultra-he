use crate::error::{Error, Result};

pub const TABLE: usize = 56;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    pub vid: u16,
    pub pid: u16,
    pub firmware: String,
    pub battery: u8,
    /// Bytes 20-21 del bloque 0x10. hec lo llama `AxialModel`.
    pub axial_model: u16,
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
    let axial_model = if payload.len() >= 22 {
        u16::from_le_bytes([payload[20], payload[21]])
    } else {
        0
    };
    Ok(DeviceInfo {
        vid,
        pid,
        firmware: format!("{}.{:02}", hundredths / 100, hundredths % 100),
        battery: payload[17],
        axial_model,
    })
}
