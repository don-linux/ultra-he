use crate::board::{load_read, tables_to_read, Board, Origin};
use crate::endpoint::{Endpoint, Io};
use crate::error::{Error, Result};
use crate::frame::{self, GET_MACRO};
use crate::macro_store;

pub fn download<T: Io>(endpoint: &mut Endpoint<T>) -> Result<Board> {
    let mut board = Board::default();
    for (cmd, len) in tables_to_read() {
        let bytes = endpoint.read_table(*cmd, *len)?;
        load_read(&mut board, *cmd, &bytes)?;
    }
    board.dongle = endpoint.io.link().kind == crate::link::Kind::Dongle;
    board.raw_macros = read_macro_image(endpoint)?;
    board.macros = crate::macro_store::decode_image(&board.raw_macros)?;
    board.macros_edited = false;
    board.origin = Origin::Device;
    board.present = crate::board::TableSet::all();
    board.capture_snapshot()?;
    Ok(board)
}

/// Escribe solo las tablas distintas a la última lectura, en el orden de hec, y las relee.
pub fn upload<T: Io>(endpoint: &mut Endpoint<T>, board: &mut Board) -> Result<()> {
    if !board.ready() {
        return Err(Error::NotReady);
    }
    board.validate()?;
    for cmd in board.command_order()? {
        let data = board.payload(cmd)?;
        if data.is_empty() {
            continue;
        }
        endpoint.write_table(cmd, &data)?;
        let back = endpoint.read_table(frame::get_for(cmd)?, data.len())?;
        if back != data {
            return Err(Error::VerifyMismatch { cmd });
        }
    }
    board.capture_snapshot()?;
    Ok(())
}

/// Carga un respaldo como cambios pendientes sobre una lectura nueva y escribe solo lo distinto.
pub fn restore<T: Io>(endpoint: &mut Endpoint<T>, backup: &Board) -> Result<Board> {
    let mut current = download(endpoint)?;
    current.overlay_present(backup)?;
    current.origin = Origin::Device;
    upload(endpoint, &mut current)?;
    Ok(current)
}

fn read_macro_image<T: Io>(endpoint: &mut Endpoint<T>) -> Result<Vec<u8>> {
    let directory = endpoint.read_table(GET_MACRO, macro_store::DIRECTORY)?;
    let mut end = macro_store::DIRECTORY;
    for slot in 0..macro_store::SLOTS {
        let start = slot * 4;
        let bytes: [u8; 4] = directory
            .get(start..start + 4)
            .ok_or(Error::Protocol("directorio de macros incompleto"))?
            .try_into()
            .map_err(|_| Error::Protocol("directorio de macros incompleto"))?;
        let offset = u32::from_le_bytes(bytes);
        if offset == 0 {
            continue;
        }
        let offset = usize::try_from(offset).map_err(|_| Error::OutOfRange {
            field: "offset de macro",
            detail: offset.to_string(),
        })?;
        if offset < macro_store::DIRECTORY {
            return Err(Error::Protocol(
                "el directorio de macros apunta dentro de sí mismo",
            ));
        }
        if offset > usize::from(u16::MAX) {
            return Err(Error::OutOfRange {
                field: "offset de macro",
                detail: offset.to_string(),
            });
        }
        let header = read_at(endpoint, GET_MACRO, offset, 4)?;
        let words = usize::from(u16::from_le_bytes([header[0], header[1]]));
        let tail = words.checked_mul(2).and_then(|body| body.checked_add(4));
        let next = tail.and_then(|tail| offset.checked_add(tail));
        let Some(next) = next else {
            return Err(Error::OutOfRange {
                field: "imagen de macros",
                detail: "el final no cabe".into(),
            });
        };
        if next > usize::from(u16::MAX) {
            return Err(Error::OutOfRange {
                field: "imagen de macros",
                detail: next.to_string(),
            });
        }
        end = end.max(next);
    }
    if end == macro_store::DIRECTORY {
        return Ok(directory);
    }
    endpoint.read_table(GET_MACRO, end)
}

fn read_at<T: Io>(
    endpoint: &mut Endpoint<T>,
    cmd: u8,
    offset: usize,
    len: usize,
) -> Result<Vec<u8>> {
    let offset_u16 = u16::try_from(offset).map_err(|_| Error::OutOfRange {
        field: "offset de macro",
        detail: offset.to_string(),
    })?;
    let frame_len = endpoint.io.link().frame_len;
    let frame = frame::build_request(frame_len, cmd, offset_u16, &vec![0u8; len], true)?;
    endpoint.write_frame(&frame)?;
    match endpoint.io.read(500)? {
        crate::endpoint::Read::Data(buf) => {
            let payload = frame::parse_reply(&buf, cmd, offset_u16)?;
            if payload.len() != len {
                return Err(Error::BadLength {
                    what: "trozo",
                    got: payload.len(),
                    expected: len,
                });
            }
            Ok(payload.to_vec())
        }
        crate::endpoint::Read::Timeout => Err(Error::Timeout),
    }
}
