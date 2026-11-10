use crate::error::{Error, Result};
use crate::frame::{self, parse_reply, prepend_report_id};
use crate::link::{self, Link};

pub enum Read {
    Timeout,
    Data(Vec<u8>),
}

pub trait Io {
    fn link(&self) -> &Link;
    fn write(&mut self, report_with_id: &[u8]) -> Result<()>;
    fn read(&mut self, timeout_ms: i32) -> Result<Read>;
}

pub struct Endpoint<T: Io> {
    pub io: T,
}

impl<T: Io> Endpoint<T> {
    pub fn new(io: T) -> Self {
        Self { io }
    }

    pub fn write_frame(&mut self, frame: &[u8]) -> Result<()> {
        link::accept_frame(self.io.link(), frame)?;
        self.io.write(&prepend_report_id(frame))
    }

    fn read_chunk(&mut self, request: &[u8]) -> Result<Vec<u8>> {
        let cmd = request[1];
        let offset = u16::from_le_bytes([request[3], request[4]]);
        for _ in 0..3 {
            self.write_frame(request)?;
            match self.io.read(500)? {
                Read::Timeout => continue,
                Read::Data(buf) => return Ok(parse_reply(&buf, cmd, offset)?.to_vec()),
            }
        }
        Err(Error::Timeout)
    }

    pub fn read_table(&mut self, cmd: u8, total: usize) -> Result<Vec<u8>> {
        let requests = frame::read_requests(self.io.link().frame_len, cmd, total)?;
        let mut out = vec![0u8; total];
        for request in requests {
            let offset = u16::from_le_bytes([request[3], request[4]]) as usize;
            let payload = self.read_chunk(&request)?;
            let end = offset + payload.len();
            if end > out.len() {
                return Err(Error::Protocol(
                    "el teclado devolvió más bytes de los pedidos",
                ));
            }
            out[offset..end].copy_from_slice(&payload);
        }
        Ok(out)
    }

    pub fn write_table(&mut self, cmd: u8, data: &[u8]) -> Result<()> {
        let frames = frame::paginate(self.io.link().frame_len, cmd, data)?;
        for frame in frames {
            self.write_frame(&frame)?;
            let offset = u16::from_le_bytes([frame[3], frame[4]]);
            match self.io.read(500) {
                Ok(Read::Data(buf)) => {
                    parse_reply(&buf, cmd, offset)?;
                }
                Ok(Read::Timeout) => return Err(Error::Timeout),
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }

    /// Lee, deja que `edit` cambie la copia y solo entonces escribe. Si la lectura falla, no hay SET.
    pub fn read_modify_write<F>(
        &mut self,
        get_cmd: u8,
        set_cmd: u8,
        total: usize,
        edit: F,
    ) -> Result<()>
    where
        F: FnOnce(Vec<u8>) -> Result<Vec<u8>>,
    {
        let current = self.read_table(get_cmd, total)?;
        let next = edit(current)?;
        if next.len() != total {
            return Err(Error::BadLength {
                what: "tabla",
                got: next.len(),
                expected: total,
            });
        }
        self.write_table(set_cmd, &next)?;
        let back = self.read_table(get_cmd, total)?;
        if back != next {
            return Err(Error::VerifyMismatch { cmd: set_cmd });
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct MockIo {
    pub link: Link,
    pub frames: Vec<Vec<u8>>,
    pub timeouts: u32,
    pub fatal: bool,
    pub tables: Vec<(u8, Vec<u8>)>,
}

impl MockIo {
    pub fn wired() -> Self {
        Self {
            link: Link::wired(),
            frames: Vec::new(),
            timeouts: 0,
            fatal: false,
            tables: Vec::new(),
        }
    }

    pub fn with_table(mut self, cmd: u8, data: Vec<u8>) -> Self {
        self.tables.push((cmd, data));
        self
    }

    fn table_payload(&self, cmd: u8, offset: usize, len: usize) -> Vec<u8> {
        let get = cmd & 0x0F | 0x10;
        let set = cmd & 0x0F | 0x20;
        let stored = self
            .tables
            .iter()
            .rev()
            .find(|(id, _)| *id == cmd || *id == get || *id == set);
        let Some((_, data)) = stored else {
            return vec![0u8; len];
        };
        let end = (offset + len).min(data.len());
        let mut out = vec![0u8; len];
        if offset < data.len() {
            let n = end - offset;
            out[..n].copy_from_slice(&data[offset..end]);
        }
        out
    }
}

impl Io for MockIo {
    fn link(&self) -> &Link {
        &self.link
    }

    fn write(&mut self, report_with_id: &[u8]) -> Result<()> {
        if report_with_id.first() != Some(&0x00) {
            return Err(Error::Protocol("el report id no es 0"));
        }
        let frame = report_with_id[1..].to_vec();
        if frame.first() == Some(&frame::REQ_MAGIC) && frame.len() > 2 && frame[2] > 0 {
            let cmd = frame[1];
            if cmd >= 0x20 {
                let offset = u16::from_le_bytes([frame[3], frame[4]]) as usize;
                let len = frame[2] as usize;
                let payload = frame[8..8 + len].to_vec();
                if let Some((_, data)) = self
                    .tables
                    .iter_mut()
                    .rev()
                    .find(|(id, _)| *id == cmd || *id == (cmd - 0x10))
                {
                    if data.len() < offset + len {
                        data.resize(offset + len, 0);
                    }
                    data[offset..offset + len].copy_from_slice(&payload);
                } else {
                    let mut data = vec![0u8; offset + len];
                    data[offset..].copy_from_slice(&payload);
                    self.tables.push((cmd, data));
                }
            }
        }
        self.frames.push(frame);
        Ok(())
    }

    fn read(&mut self, _timeout_ms: i32) -> Result<Read> {
        if self.fatal {
            return Err(Error::Io("lectura cortada".into()));
        }
        if self.timeouts > 0 {
            self.timeouts -= 1;
            return Ok(Read::Timeout);
        }
        let Some(request) = self.frames.last() else {
            return Ok(Read::Timeout);
        };
        let cmd = request[1];
        let len = request[2] as usize;
        let offset = u16::from_le_bytes([request[3], request[4]]) as usize;
        let payload = self.table_payload(cmd, offset, len);
        let mut reply = vec![0u8; request.len()];
        reply[0] = frame::REP_MAGIC;
        reply[1] = cmd;
        reply[2] = payload.len() as u8;
        reply[3] = request[3];
        reply[4] = request[4];
        reply[6] = request[6];
        reply[8..8 + payload.len()].copy_from_slice(&payload);
        Ok(Read::Data(reply))
    }
}
