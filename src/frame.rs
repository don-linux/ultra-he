use crate::error::{Error, Result};

pub const REQ_MAGIC: u8 = 0xAA;
pub const REP_MAGIC: u8 = 0x55;
pub const WIRED_FRAME: usize = 64;
pub const DONGLE_FRAME: usize = 32;
pub const HEADER: usize = 8;

pub const GET_INFO: u8 = 0x10;
pub const GET_GLOBALS: u8 = 0x11;
pub const GET_KEYS: u8 = 0x12;
pub const GET_LIGHT: u8 = 0x13;
pub const GET_COLOR: u8 = 0x14;
pub const GET_MACRO: u8 = 0x15;
pub const GET_FN: u8 = 0x16;
pub const GET_ACTUATION: u8 = 0x17;
pub const GET_DKS: u8 = 0x18;
pub const SET_GLOBALS: u8 = 0x21;
pub const SET_KEYS: u8 = 0x22;
pub const SET_LIGHT: u8 = 0x23;
pub const SET_COLOR: u8 = 0x24;
pub const SET_MACRO: u8 = 0x25;
pub const SET_FN: u8 = 0x26;
pub const SET_ACTUATION: u8 = 0x27;
pub const SET_DKS: u8 = 0x28;

const FORBIDDEN: [u8; 4] = [0x0F, 0x4F, 0x64, 0x65];

pub fn ensure_allowed(cmd: u8) -> Result<()> {
    if FORBIDDEN.contains(&cmd) {
        Err(Error::ForbiddenCommand(cmd))
    } else {
        Ok(())
    }
}

/// Lectura que corresponde a un `SET`. Solo los ocho pares que hekit escribe.
pub fn get_for(set: u8) -> Result<u8> {
    let get = match set {
        SET_GLOBALS => GET_GLOBALS,
        SET_KEYS => GET_KEYS,
        SET_LIGHT => GET_LIGHT,
        SET_COLOR => GET_COLOR,
        SET_MACRO => GET_MACRO,
        SET_FN => GET_FN,
        SET_ACTUATION => GET_ACTUATION,
        SET_DKS => GET_DKS,
        _ => return Err(Error::Protocol("el comando no tiene lectura pareja")),
    };
    Ok(get)
}

fn offset_u16(offset: usize) -> Result<u16> {
    u16::try_from(offset).map_err(|_| Error::OutOfRange {
        field: "offset",
        detail: format!("{offset}, máximo {}", u16::MAX),
    })
}

pub fn chunk_size(frame_len: usize) -> usize {
    frame_len.saturating_sub(HEADER)
}

/// Arma una trama de `frame_len` bytes. El report id 0 lo agrega el transporte.
pub fn build_request(
    frame_len: usize,
    cmd: u8,
    offset: u16,
    payload: &[u8],
    last: bool,
) -> Result<Vec<u8>> {
    ensure_allowed(cmd)?;
    if frame_len != WIRED_FRAME && frame_len != DONGLE_FRAME {
        return Err(Error::Protocol("largo de trama desconocido"));
    }
    let max = chunk_size(frame_len);
    if payload.len() > max {
        return Err(Error::BadLength {
            what: "trozo",
            got: payload.len(),
            expected: max,
        });
    }
    let mut frame = vec![0u8; frame_len];
    frame[0] = REQ_MAGIC;
    frame[1] = cmd;
    frame[2] = payload.len() as u8;
    let off = offset.to_le_bytes();
    frame[3] = off[0];
    frame[4] = off[1];
    frame[6] = u8::from(last);
    frame[8..8 + payload.len()].copy_from_slice(payload);
    Ok(frame)
}

pub fn paginate(frame_len: usize, cmd: u8, data: &[u8]) -> Result<Vec<Vec<u8>>> {
    ensure_allowed(cmd)?;
    if data.is_empty() {
        return Ok(vec![build_request(frame_len, cmd, 0, &[], true)?]);
    }
    let size = chunk_size(frame_len);
    let mut out = Vec::new();
    let mut offset = 0usize;
    while offset < data.len() {
        let end = (offset + size).min(data.len());
        let last = end == data.len();
        out.push(build_request(
            frame_len,
            cmd,
            offset_u16(offset)?,
            &data[offset..end],
            last,
        )?);
        offset = end;
    }
    Ok(out)
}

/// Peticiones de lectura. El byte de largo pide el trozo; el payload va en cero.
pub fn read_requests(frame_len: usize, cmd: u8, total: usize) -> Result<Vec<Vec<u8>>> {
    ensure_allowed(cmd)?;
    if total == 0 {
        return Ok(vec![build_request(frame_len, cmd, 0, &[], true)?]);
    }
    let size = chunk_size(frame_len);
    let mut out = Vec::new();
    let mut offset = 0usize;
    while offset < total {
        let n = (total - offset).min(size);
        let last = offset + n == total;
        out.push(build_request(
            frame_len,
            cmd,
            offset_u16(offset)?,
            &vec![0u8; n],
            last,
        )?);
        offset += n;
    }
    Ok(out)
}

pub fn prepend_report_id(frame: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(frame.len() + 1);
    out.push(0x00);
    out.extend_from_slice(frame);
    out
}

/// Devuelve el payload del trozo. Acepta un `0x00` inicial que algunos hidapi dejan puesto.
pub fn parse_reply(buf: &[u8], expect_cmd: u8, expect_offset: u16) -> Result<&[u8]> {
    let buf = if buf.first() == Some(&0x00) && buf.get(1) == Some(&REP_MAGIC) {
        &buf[1..]
    } else {
        buf
    };
    if buf.first() != Some(&REP_MAGIC) {
        return Err(Error::Protocol("la respuesta no empieza con 0x55"));
    }
    if buf.len() < HEADER {
        return Err(Error::Protocol("respuesta demasiado corta"));
    }
    if buf[1] != expect_cmd {
        return Err(Error::Protocol("la respuesta trae otro comando"));
    }
    let offset = u16::from_le_bytes([buf[3], buf[4]]);
    if offset != expect_offset {
        return Err(Error::Protocol("la respuesta trae otro offset"));
    }
    let len = buf[2] as usize;
    let end = HEADER + len;
    if buf.len() < end {
        return Err(Error::Protocol("la respuesta no trae el payload anunciado"));
    }
    Ok(&buf[HEADER..end])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forbidden_commands_never_build() {
        for cmd in [0x0F_u8, 0x4F, 0x64, 0x65] {
            assert!(matches!(
                build_request(WIRED_FRAME, cmd, 0, &[], true),
                Err(Error::ForbiddenCommand(_))
            ));
        }
    }

    #[test]
    fn offset_is_little_endian_and_only_the_last_chunk_is_flagged() {
        let frames = paginate(WIRED_FRAME, SET_GLOBALS, &[0u8; 64]).unwrap();
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0][3], 0x00);
        assert_eq!(frames[0][4], 0x00);
        assert_eq!(frames[0][6], 0);
        assert_eq!(frames[1][3], 0x38);
        assert_eq!(frames[1][4], 0x00);
        assert_eq!(frames[1][2], 8);
        assert_eq!(frames[1][6], 1);
    }

    #[test]
    fn get_for_pairs_each_set_and_rejects_the_rest() {
        assert_eq!(get_for(SET_GLOBALS).unwrap(), GET_GLOBALS);
        assert_eq!(get_for(SET_KEYS).unwrap(), GET_KEYS);
        assert_eq!(get_for(SET_LIGHT).unwrap(), GET_LIGHT);
        assert_eq!(get_for(SET_COLOR).unwrap(), GET_COLOR);
        assert_eq!(get_for(SET_MACRO).unwrap(), GET_MACRO);
        assert_eq!(get_for(SET_FN).unwrap(), GET_FN);
        assert_eq!(get_for(SET_ACTUATION).unwrap(), GET_ACTUATION);
        assert_eq!(get_for(SET_DKS).unwrap(), GET_DKS);
        assert!(get_for(GET_LIGHT).is_err());
        assert!(get_for(0x0F).is_err());
    }

    #[test]
    fn paginate_rejects_an_offset_past_u16() {
        let chunk = chunk_size(WIRED_FRAME);
        let data = vec![0u8; chunk * (usize::from(u16::MAX) / chunk + 1) + 1];
        assert!(paginate(WIRED_FRAME, SET_KEYS, &data).is_err());
    }
}
