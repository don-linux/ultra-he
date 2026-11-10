use crate::error::{Error, Result};
use crate::frame::{DONGLE_FRAME, WIRED_FRAME};

pub const VID: u16 = 0x0C45;
pub const PID_WIRED: u16 = 0x80A2;
pub const PID_DONGLE: u16 = 0xFEFE;
pub const PID_ALT: u16 = 0x80B2;
pub const PAGE_WIRED: u16 = 0xFF68;
pub const PAGE_DONGLE: u16 = 0xFF60;
pub const USAGE: u16 = 0x61;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Wired,
    Dongle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    pub kind: Kind,
    pub frame_len: usize,
    pub product: String,
}

impl Link {
    pub fn wired() -> Self {
        Self {
            kind: Kind::Wired,
            frame_len: WIRED_FRAME,
            product: "KEYBOARD MINI 60 HE PRO".into(),
        }
    }

    pub fn dongle() -> Self {
        Self {
            kind: Kind::Dongle,
            frame_len: DONGLE_FRAME,
            product: "MINI 60 HE PRO Dongle".into(),
        }
    }

    pub fn label(&self) -> &'static str {
        match self.kind {
            Kind::Wired => "Cable USB",
            Kind::Dongle => "Receptor 2.4 GHz",
        }
    }
}

pub fn classify(vid: u16, pid: u16, usage_page: u16, usage: u16, product: &str) -> Result<Link> {
    if vid != VID {
        return Err(Error::NoDevice);
    }
    if pid == PID_ALT {
        return Err(Error::RefusedPid);
    }
    if pid == PID_WIRED && usage_page == PAGE_WIRED && usage == USAGE {
        return Ok(Link {
            product: product.to_string(),
            ..Link::wired()
        });
    }
    if pid == PID_DONGLE && usage_page == PAGE_DONGLE && usage == USAGE {
        if !product.to_ascii_uppercase().contains("MINI 60 HE PRO") {
            return Err(Error::WrongInterface);
        }
        return Ok(Link {
            product: product.to_string(),
            ..Link::dongle()
        });
    }
    Err(Error::WrongInterface)
}

pub fn prefer(found: &[Link]) -> Result<Link> {
    if let Some(link) = found.iter().find(|link| link.kind == Kind::Wired) {
        return Ok(link.clone());
    }
    if let Some(link) = found.iter().find(|link| link.kind == Kind::Dongle) {
        return Ok(link.clone());
    }
    Err(Error::NoDevice)
}

pub fn accept_frame(link: &Link, frame: &[u8]) -> Result<()> {
    if frame.len() != link.frame_len {
        return Err(Error::BadLength {
            what: "trama",
            got: frame.len(),
            expected: link.frame_len,
        });
    }
    Ok(())
}
