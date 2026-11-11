use crate::actuation;
use crate::board::{load_read, tables_to_read, Board, Origin};
use crate::endpoint::{Endpoint, Io};
use crate::error::{Error, Result};
use crate::frame::{self, GET_MACRO};
use crate::globals;

pub fn download<T: Io>(endpoint: &mut Endpoint<T>) -> Result<Board> {
    let mut board = Board::default();
    for (cmd, len) in tables_to_read() {
        let bytes = endpoint.read_table(*cmd, *len)?;
        load_read(&mut board, *cmd, &bytes)?;
    }
    board.dongle = endpoint.io.link().kind == crate::link::Kind::Dongle;
    board.raw_macros = read_macro_image(endpoint)?;
    board.macros_edited = false;
    board.origin = Origin::Device;
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
        let back = endpoint.read_table(cmd - 0x10, data.len())?;
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
    current.globals = backup.globals.clone();
    current.actuation = backup.actuation;
    current.choice = backup.choice;
    current.base = backup.base;
    current.fn_layer = backup.fn_layer;
    current.dks = backup.dks;
    current.macros = backup.macros.clone();
    current.raw_macros = backup.raw_macros.clone();
    current.macros_edited = backup.macros_edited;
    current.lighting = backup.lighting;
    current.colors = backup.colors;
    current.origin = Origin::Device;
    upload(endpoint, &mut current)?;
    Ok(current)
}

fn read_macro_image<T: Io>(endpoint: &mut Endpoint<T>) -> Result<Vec<u8>> {
    let directory = endpoint.read_table(GET_MACRO, 400)?;
    let mut end = 400usize;
    for slot in 0..100 {
        let start = slot * 4;
        let offset = u32::from_le_bytes(directory[start..start + 4].try_into().unwrap()) as usize;
        if offset == 0 {
            continue;
        }
        if offset < 400 {
            return Err(Error::Protocol(
                "el directorio de macros apunta dentro de sí mismo",
            ));
        }
        let header = read_at(endpoint, GET_MACRO, offset, 4)?;
        let words = u16::from_le_bytes([header[0], header[1]]) as usize;
        end = end.max(offset + 4 + words * 2);
    }
    if end == 400 {
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
    let frame_len = endpoint.io.link().frame_len;
    let frame = frame::build_request(frame_len, cmd, offset as u16, &vec![0u8; len], true)?;
    endpoint.write_frame(&frame)?;
    match endpoint.io.read(500)? {
        crate::endpoint::Read::Data(buf) => {
            Ok(frame::parse_reply(&buf, cmd, offset as u16)?.to_vec())
        }
        crate::endpoint::Read::Timeout => Err(Error::Timeout),
    }
}

pub fn actuation_table_len() -> usize {
    actuation::TABLE
}

pub fn globals_len() -> usize {
    globals::TABLE
}
