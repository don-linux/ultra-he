use crate::actuation;
use crate::board::{self, load_read, tables_to_read, Board, Origin};
use crate::dks;
use crate::endpoint::{Endpoint, Io};
use crate::error::{Error, Result};
use crate::frame::{
    self, GET_MACRO, SET_ACTUATION, SET_COLOR, SET_DKS, SET_FN, SET_GLOBALS, SET_KEYS, SET_LIGHT,
};
use crate::globals;
use crate::keys;
use crate::lighting;

pub fn download<T: Io>(endpoint: &mut Endpoint<T>) -> Result<Board> {
    let mut board = Board::default();
    for (cmd, len) in tables_to_read() {
        let bytes = endpoint.read_table(*cmd, *len)?;
        load_read(&mut board, *cmd, &bytes)?;
    }
    board.raw_macros = read_macro_image(endpoint)?;
    board.macros_edited = false;
    board.origin = Origin::Device;
    Ok(board)
}

pub fn upload<T: Io>(endpoint: &mut Endpoint<T>, board: &Board) -> Result<()> {
    if !board.ready() {
        return Err(Error::NotReady);
    }
    board.validate()?;
    for cmd in board.command_order() {
        let data = payload(board, cmd)?;
        if data.is_empty() {
            continue;
        }
        endpoint.write_table(cmd, &data)?;
        let back = endpoint.read_table(cmd - 0x10, data.len())?;
        if back != data {
            return Err(Error::VerifyMismatch { cmd });
        }
    }
    Ok(())
}

fn payload(board: &Board, cmd: u8) -> Result<Vec<u8>> {
    Ok(match cmd {
        SET_GLOBALS => board.globals.raw.to_vec(),
        SET_ACTUATION => board.actuation_bytes()?.to_vec(),
        SET_DKS => dks::encode_table(&board.dks).to_vec(),
        frame::SET_MACRO => macro_payload(board)?,
        SET_KEYS => keys::base_write_bytes(&board.base)?,
        SET_FN => keys::fn_write_bytes(&board.fn_layer),
        SET_LIGHT => board.lighting.encode()?.to_vec(),
        SET_COLOR => lighting::encode_colors(&board.colors).to_vec(),
        _ => return Err(Error::Protocol("comando de escritura desconocido")),
    })
}

fn macro_payload(board: &Board) -> Result<Vec<u8>> {
    if board.macros_edited || board.origin == Origin::File {
        return board::macro_image(board);
    }
    Ok(board.raw_macros.clone())
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
