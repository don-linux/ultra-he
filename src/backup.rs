use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::actuation;
use crate::board::{Board, Origin, TableSet};
use crate::dks;
use crate::error::{Error, Result};
use crate::globals::Globals;
use crate::info::DeviceInfo;
use crate::keys;
use crate::lighting::{self, Lighting};
use crate::link::PID_WIRED;
use crate::switches::SwitchChoice;

const FORMAT: &str = "hekit-backup";

#[derive(Debug, Serialize, Deserialize)]
struct File {
    format: String,
    saved_unix: u64,
    firmware: String,
    axial_model: u16,
    default_axis: u8,
    dongle: bool,
    globals: Vec<u8>,
    actuation: Vec<u8>,
    choices: Vec<String>,
    base: Vec<u8>,
    fn_layer: Vec<u8>,
    dks: Vec<u8>,
    macros: Vec<u8>,
    light: Vec<u8>,
    colors: Vec<u8>,
}

pub fn save(board: &Board) -> Result<PathBuf> {
    let dir = backup_dir()?;
    fs::create_dir_all(&dir).map_err(|error| Error::Io(error.to_string()))?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let path = dir.join(format!("backup-{stamp}.json"));
    save_to(board, &path)?;
    Ok(path)
}

pub fn save_to(board: &Board, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| Error::Io(error.to_string()))?;
    }
    let file = File {
        format: FORMAT.into(),
        saved_unix: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0),
        firmware: board
            .info
            .as_ref()
            .map(|info| info.firmware.clone())
            .unwrap_or_default(),
        axial_model: board
            .info
            .as_ref()
            .map(|info| info.axial_model)
            .unwrap_or(0),
        default_axis: board.default_axis,
        dongle: board.dongle,
        globals: board.globals.raw.to_vec(),
        actuation: actuation::encode_table(&board.actuation).to_vec(),
        choices: board
            .choice
            .iter()
            .map(|choice| choice.profile_key())
            .collect(),
        base: keys::encode_table(&board.base).to_vec(),
        fn_layer: keys::fn_write_bytes(&board.fn_layer),
        dks: dks::encode_table(&board.dks).to_vec(),
        macros: if board.macros_edited {
            crate::board::macro_image(board)?
        } else {
            board.raw_macros.clone()
        },
        light: board.lighting.encode()?.to_vec(),
        colors: lighting::encode_colors(&board.colors).to_vec(),
    };
    let text = serde_json::to_string(&file).map_err(|error| Error::Json(error.to_string()))?;
    fs::write(path, text).map_err(|error| Error::Io(error.to_string()))
}

pub fn load(path: &Path) -> Result<Board> {
    let text = fs::read_to_string(path).map_err(|error| Error::Io(error.to_string()))?;
    parse(&text)
}

pub fn parse(text: &str) -> Result<Board> {
    let file: File = serde_json::from_str(text).map_err(|error| Error::Json(error.to_string()))?;
    if file.format != FORMAT {
        return Err(Error::Json("el archivo no es un respaldo de hekit".into()));
    }
    let mut board = Board {
        origin: Origin::File,
        dongle: file.dongle,
        default_axis: file.default_axis,
        info: Some(DeviceInfo {
            vid: 0x0C45,
            pid: PID_WIRED,
            firmware: file.firmware,
            battery: 0,
            axial_model: file.axial_model,
        }),
        globals: Globals::from_bytes(&file.globals)?,
        ..Board::default()
    };
    board.actuation = actuation::decode_table(&file.actuation)?;
    board.base = keys::decode_table(&file.base)?;
    board.fn_layer = keys::decode_table(&file.fn_layer)?;
    board.dks = dks::decode_table(&file.dks)?;
    board.lighting = Lighting::decode(&file.light)?;
    board.colors = lighting::decode_colors(&file.colors)?;
    board.raw_macros = file.macros;
    for (index, key) in file.choices.iter().enumerate() {
        if index >= actuation::RECORDS {
            break;
        }
        board.choice[index] = SwitchChoice::parse_profile(key)?;
    }
    board.present = TableSet::all();
    Ok(board)
}

pub fn backup_dir() -> Result<PathBuf> {
    if let Some(dir) = std::env::var_os("HEKIT_BACKUP_DIR") {
        return Ok(PathBuf::from(dir));
    }
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local").join("share"))
        })
        .ok_or_else(|| Error::Io("no se encontró el directorio de datos".into()))?;
    Ok(base.join("hekit").join("backups"))
}

pub fn list() -> Result<Vec<PathBuf>> {
    let dir = backup_dir()?;
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut paths = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|error| Error::Io(error.to_string()))? {
        let entry = entry.map_err(|error| Error::Io(error.to_string()))?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}
