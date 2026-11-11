use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::actuation;
use crate::board::Board;
use crate::error::{Error, Result};
use crate::layout;
use crate::switches::SwitchChoice;

/// Teclas que el usuario marcó como Jade Ultra. El firmware solo guarda el id 2.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SwitchMap {
    ultra: Vec<u16>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct File {
    ultra: Vec<u16>,
}

impl SwitchMap {
    pub fn contains(&self, index: u16) -> bool {
        self.ultra.contains(&index)
    }

    pub fn set_physical_all(&mut self) {
        self.ultra = layout::keys().into_iter().map(|key| key.index).collect();
    }

    pub fn mark(&mut self, indexes: impl IntoIterator<Item = u16>) {
        for index in indexes {
            if !self.ultra.contains(&index) {
                self.ultra.push(index);
            }
        }
    }

    pub fn unmark(&mut self, indexes: impl IntoIterator<Item = u16>) {
        let drop: Vec<u16> = indexes.into_iter().collect();
        self.ultra.retain(|index| !drop.contains(index));
    }

    pub fn clear(&mut self) {
        self.ultra.clear();
    }

    /// Si la tecla está en el mapa y el teclado trae id 2, se muestra como Jade Ultra.
    pub fn overlay(&self, board: &mut Board) {
        for index in &self.ultra {
            let index = *index as usize;
            if index >= actuation::RECORDS {
                continue;
            }
            if board.actuation[index].axis_type == 2 {
                board.choice[index] = SwitchChoice::JadeUltra;
            }
        }
    }

    /// Marcadas como Ultra cuyo byte en el teclado no es 2.
    pub fn mismatches(&self, board: &Board) -> Vec<u16> {
        self.ultra
            .iter()
            .copied()
            .filter(|index| {
                let slot = *index as usize;
                slot < actuation::RECORDS && board.actuation[slot].axis_type != 2
            })
            .collect()
    }

    pub fn load() -> Result<Self> {
        let path = config_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        Self::load_from(&path)
    }

    pub fn load_from(path: &Path) -> Result<Self> {
        let text = fs::read_to_string(path).map_err(|error| Error::Io(error.to_string()))?;
        let file: File =
            serde_json::from_str(&text).map_err(|error| Error::Json(error.to_string()))?;
        Ok(Self { ultra: file.ultra })
    }

    pub fn save(&self) -> Result<PathBuf> {
        let path = config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| Error::Io(error.to_string()))?;
        }
        self.save_to(&path)?;
        Ok(path)
    }

    pub fn save_to(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| Error::Io(error.to_string()))?;
        }
        let file = File {
            ultra: self.ultra.clone(),
        };
        let text =
            serde_json::to_string_pretty(&file).map_err(|error| Error::Json(error.to_string()))?;
        fs::write(path, text).map_err(|error| Error::Io(error.to_string()))
    }
}

pub fn config_path() -> Result<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .ok_or_else(|| Error::Io("no se encontró el directorio de configuración".into()))?;
    Ok(base.join("hekit").join("switch-map.json"))
}
