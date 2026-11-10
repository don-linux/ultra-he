use std::ffi::CString;

use hidapi::{HidApi, HidDevice};

use crate::error::{Error, Result};
use crate::link::{self, Link};

pub struct Device {
    _api: HidApi,
    device: HidDevice,
    pub link: Link,
}

impl Device {
    pub fn open() -> Result<Self> {
        let api = HidApi::new().map_err(|error| Error::Io(error.to_string()))?;
        let mut found: Vec<(CString, Link)> = Vec::new();
        for info in api.device_list() {
            let product = info.product_string().unwrap_or("");
            if let Ok(link) = link::classify(
                info.vendor_id(),
                info.product_id(),
                info.usage_page(),
                info.usage(),
                product,
            ) {
                found.push((info.path().to_owned(), link));
            }
        }
        let links: Vec<Link> = found.iter().map(|(_, link)| link.clone()).collect();
        let chosen = link::prefer(&links)?;
        let path = found
            .into_iter()
            .find(|(_, link)| link.kind == chosen.kind)
            .map(|(path, _)| path)
            .ok_or(Error::NoDevice)?;
        let device = api
            .open_path(&path)
            .map_err(|error| Error::Io(error.to_string()))?;
        Ok(Self {
            _api: api,
            device,
            link: chosen,
        })
    }

    pub fn write(&self, report: &[u8]) -> Result<()> {
        let wrote = self
            .device
            .write(report)
            .map_err(|error| Error::Io(error.to_string()))?;
        if wrote < report.len() {
            return Err(Error::Io(format!(
                "se escribieron {wrote} de {} bytes",
                report.len()
            )));
        }
        Ok(())
    }

    pub fn read(&self, timeout_ms: i32) -> Result<crate::endpoint::Read> {
        let mut buf = [0u8; 128];
        match self.device.read_timeout(&mut buf, timeout_ms) {
            Ok(0) => Ok(crate::endpoint::Read::Timeout),
            Ok(n) => Ok(crate::endpoint::Read::Data(buf[..n].to_vec())),
            Err(error) => Err(Error::Io(error.to_string())),
        }
    }
}

impl crate::endpoint::Io for Device {
    fn link(&self) -> &Link {
        &self.link
    }

    fn write(&mut self, report_with_id: &[u8]) -> Result<()> {
        Device::write(self, report_with_id)
    }

    fn read(&mut self, timeout_ms: i32) -> Result<crate::endpoint::Read> {
        Device::read(self, timeout_ms)
    }
}
