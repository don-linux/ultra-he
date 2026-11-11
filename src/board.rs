use crate::actuation::{self, Record};
use crate::dks::{self, Slot};
use crate::error::{Error, Result};
use crate::frame::{
    self, GET_ACTUATION, GET_COLOR, GET_DKS, GET_FN, GET_GLOBALS, GET_INFO, GET_KEYS, GET_LIGHT,
    SET_ACTUATION, SET_COLOR, SET_DKS, SET_FN, SET_GLOBALS, SET_KEYS, SET_LIGHT,
};
use crate::globals::Globals;
use crate::info::{self, DeviceInfo};
use crate::keys::{self, KeyRecord};
use crate::lighting::{self, Lighting};
use crate::macro_store::{self, Event};
use crate::switches::{self, SwitchChoice};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    Empty,
    Device,
    File,
}

/// Tablas que venían en un archivo. Una importación copia solo estas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableSet {
    pub actuation: bool,
    pub base: bool,
    pub fn_layer: bool,
    pub dks: bool,
    pub globals: bool,
    pub dead_zone: bool,
    pub lighting: bool,
    pub colors: bool,
    pub macros: bool,
}

impl TableSet {
    pub const fn none() -> Self {
        Self {
            actuation: false,
            base: false,
            fn_layer: false,
            dks: false,
            globals: false,
            dead_zone: false,
            lighting: false,
            colors: false,
            macros: false,
        }
    }

    pub const fn all() -> Self {
        Self {
            actuation: true,
            base: true,
            fn_layer: true,
            dks: true,
            globals: true,
            dead_zone: false,
            lighting: true,
            colors: true,
            macros: true,
        }
    }
}

/// Copia de las tablas tal como se escribirían. Sirve para saber qué cambió.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub globals: Vec<u8>,
    pub actuation: Vec<u8>,
    pub dks: Vec<u8>,
    pub macros: Vec<u8>,
    pub base: Vec<u8>,
    pub fn_layer: Vec<u8>,
    pub light: Vec<u8>,
    pub colors: Vec<u8>,
}

impl Snapshot {
    fn bytes(&self, cmd: u8) -> &[u8] {
        match cmd {
            SET_GLOBALS => &self.globals,
            SET_ACTUATION => &self.actuation,
            SET_DKS => &self.dks,
            frame::SET_MACRO => &self.macros,
            SET_KEYS => &self.base,
            SET_FN => &self.fn_layer,
            SET_LIGHT => &self.light,
            SET_COLOR => &self.colors,
            _ => &[],
        }
    }
}

#[derive(Debug, Clone)]
pub struct Board {
    pub origin: Origin,
    pub info: Option<DeviceInfo>,
    /// True cuando la última lectura fue por el receptor. Cambia el tope del eje 1.
    pub dongle: bool,
    /// Eje que hec usa cuando el byte de la tecla es 0.
    pub default_axis: u8,
    pub snapshot: Option<Snapshot>,
    pub globals: Globals,
    pub actuation: [Record; actuation::RECORDS],
    pub choice: [SwitchChoice; actuation::RECORDS],
    pub base: [KeyRecord; keys::SLOTS],
    pub fn_layer: [KeyRecord; keys::SLOTS],
    pub dks: [Slot; dks::SLOTS],
    pub macros: Vec<Vec<Event>>,
    pub raw_macros: Vec<u8>,
    pub macros_edited: bool,
    pub lighting: Lighting,
    pub colors: [[u8; 3]; lighting::COLOR_RECORDS],
    pub present: TableSet,
    /// La lectura terminó, pero una tabla no se pudo re-codificar.
    pub warning: Option<String>,
}

impl Default for Board {
    fn default() -> Self {
        Self {
            origin: Origin::Empty,
            info: None,
            dongle: false,
            default_axis: switches::FALLBACK_AXIS,
            snapshot: None,
            globals: Globals::default(),
            actuation: [Record::default(); actuation::RECORDS],
            choice: [SwitchChoice::Axis(6); actuation::RECORDS],
            base: [KeyRecord::default(); keys::SLOTS],
            fn_layer: [KeyRecord::default(); keys::SLOTS],
            dks: [Slot::default(); dks::SLOTS],
            macros: Vec::new(),
            raw_macros: Vec::new(),
            macros_edited: false,
            lighting: Lighting::default(),
            colors: [[0x90, 0xEE, 0x90]; lighting::COLOR_RECORDS],
            present: TableSet::none(),
            warning: None,
        }
    }
}

impl Board {
    pub fn ready(&self) -> bool {
        self.origin != Origin::Empty
    }

    pub fn cap_of(&self, choice: SwitchChoice) -> Result<u16> {
        choice.cap(self.dongle, self.default_axis)
    }

    /// Cambia el eje y, si el punto actual no cabe, lo baja al tope nuevo.
    pub fn assign_switch(&mut self, index: usize, choice: SwitchChoice) -> Result<()> {
        let max = self.cap_of(choice)?;
        let record = self.slot_mut(index)?;
        if record.trip > max {
            record.trip = max;
        }
        if record.rt_press > max {
            record.rt_press = max;
        }
        if record.rt_release > max {
            record.rt_release = max;
        }
        record.axis_type = choice.axis_byte();
        self.choice[index] = choice;
        Ok(())
    }

    pub fn set_switch(&mut self, index: usize, choice: SwitchChoice) -> Result<()> {
        let max = self.cap_of(choice)?;
        let record = self.slot_mut(index)?;
        if record.trip > max || record.rt_press > max || record.rt_release > max {
            return Err(Error::OutOfRange {
                field: "eje",
                detail: "baja la actuación o el rapid trigger antes de cambiar a un tope menor"
                    .into(),
            });
        }
        record.axis_type = choice.axis_byte();
        self.choice[index] = choice;
        Ok(())
    }

    pub fn set_trip(&mut self, index: usize, trip: u16) -> Result<()> {
        let choice = self.choice[index];
        let max = self.cap_of(choice)?;
        let record = self.slot_mut(index)?;
        let flags = record.flags;
        actuation::set_trip_capped(record, choice, trip, max)?;
        record.flags = flags;
        Ok(())
    }

    pub fn set_rt_press(&mut self, index: usize, value: u16) -> Result<()> {
        let choice = self.choice[index];
        let max = self.cap_of(choice)?;
        let record = self.slot_mut(index)?;
        let flags = record.flags;
        actuation::set_rt_press_capped(record, choice, value, max)?;
        record.flags = flags;
        Ok(())
    }

    pub fn set_rt_release(&mut self, index: usize, value: u16) -> Result<()> {
        let choice = self.choice[index];
        let max = self.cap_of(choice)?;
        let press = record_press(self, index)?;
        let record = self.slot_mut(index)?;
        let flags = record.flags;
        actuation::set_rt_release_capped(record, choice, value, max)?;
        record.rt_press = press;
        record.flags = flags;
        Ok(())
    }

    pub fn set_rt_enabled(&mut self, index: usize, enabled: bool) -> Result<()> {
        let record = self.slot_mut(index)?;
        actuation::set_rt_enabled(record, enabled);
        Ok(())
    }

    pub fn set_full_travel(&mut self, index: usize, enabled: bool) -> Result<()> {
        let record = self.slot_mut(index)?;
        actuation::set_full_travel(record, enabled);
        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        for (index, record) in self.actuation.iter().enumerate() {
            let choice = self.choice[index];
            let max = self.cap_of(choice)?;
            actuation::validate_record(choice, record, max)?;
        }
        // La tabla DKS tiene 64 ranuras. La 63 no se puede asignar a una tecla
        // nueva, pero el firmware la devuelve y hay que conservarla.
        Ok(())
    }

    /// Copia las tablas que el archivo traía. El snapshot y el enlace se quedan.
    pub fn overlay_present(&mut self, incoming: &Board) -> Result<()> {
        if incoming.present.globals {
            self.globals = incoming.globals.clone();
        } else if incoming.present.dead_zone {
            self.globals.set_top_dead(incoming.globals.top_dead_cmm())?;
            self.globals
                .set_bottom_dead(incoming.globals.bottom_dead_cmm())?;
        }
        if incoming.present.actuation {
            self.actuation = incoming.actuation;
            self.choice = incoming.choice;
        }
        if incoming.present.base {
            self.base = incoming.base;
        }
        if incoming.present.fn_layer {
            self.fn_layer = incoming.fn_layer;
        }
        if incoming.present.dks {
            self.dks = incoming.dks;
        }
        if incoming.present.lighting {
            self.lighting = incoming.lighting;
        }
        if incoming.present.colors {
            self.colors = incoming.colors;
        }
        if incoming.present.macros {
            self.macros = incoming.macros.clone();
            self.raw_macros = incoming.raw_macros.clone();
            self.macros_edited = incoming.macros_edited;
        }
        Ok(())
    }

    /// Orden de hec, solo con las tablas distintas a la última lectura.
    /// Sin lectura previa se consideran todas sucias.
    pub fn command_order(&self) -> Result<Vec<u8>> {
        let candidates = [
            SET_KEYS,
            SET_FN,
            SET_ACTUATION,
            SET_DKS,
            frame::SET_MACRO,
            SET_GLOBALS,
            SET_LIGHT,
        ];
        let Some(snapshot) = &self.snapshot else {
            let mut order = candidates.to_vec();
            if self.lighting.mode == 20 {
                order.push(SET_COLOR);
            }
            return Ok(order);
        };
        let mut order = Vec::new();
        for cmd in candidates {
            if cmd == SET_LIGHT {
                if let Ok(bytes) = self.lighting.encode() {
                    if bytes.as_slice() != snapshot.light {
                        order.push(SET_LIGHT);
                    }
                }
                continue;
            }
            let data = self.payload(cmd)?;
            if cmd == frame::SET_MACRO && data.is_empty() {
                continue;
            }
            if data != snapshot.bytes(cmd) {
                order.push(cmd);
            }
        }
        if self.lighting.mode == 20 {
            let colors = self.payload(SET_COLOR)?;
            if colors != snapshot.colors {
                order.push(SET_COLOR);
            }
        }
        Ok(order)
    }

    pub fn payload(&self, cmd: u8) -> Result<Vec<u8>> {
        Ok(match cmd {
            SET_GLOBALS => self.globals.raw.to_vec(),
            SET_ACTUATION => self.actuation_bytes()?.to_vec(),
            SET_DKS => dks::encode_table(&self.dks).to_vec(),
            frame::SET_MACRO => {
                if self.macros_edited {
                    macro_image(self)?
                } else {
                    self.raw_macros.clone()
                }
            }
            SET_KEYS => keys::base_write_bytes(&self.base)?,
            SET_FN => keys::fn_write_bytes(&self.fn_layer),
            SET_LIGHT => self.lighting.encode()?.to_vec(),
            SET_COLOR => lighting::encode_colors(&self.colors).to_vec(),
            _ => return Err(Error::Protocol("comando de escritura desconocido")),
        })
    }

    pub fn capture_snapshot(&mut self) -> Result<()> {
        let light = match self.lighting.encode() {
            Ok(bytes) => {
                self.warning = None;
                bytes.to_vec()
            }
            Err(error) => {
                self.warning = Some(error.to_string());
                self.lighting.raw.to_vec()
            }
        };
        let macros = match self.payload(frame::SET_MACRO)? {
            data if data.is_empty() => self
                .snapshot
                .as_ref()
                .map(|snapshot| snapshot.macros.clone())
                .unwrap_or_default(),
            data => data,
        };
        self.snapshot = Some(Snapshot {
            globals: self.payload(SET_GLOBALS)?,
            actuation: self.payload(SET_ACTUATION)?,
            dks: self.payload(SET_DKS)?,
            macros,
            base: self.payload(SET_KEYS)?,
            fn_layer: self.payload(SET_FN)?,
            light,
            colors: self.payload(SET_COLOR)?,
        });
        Ok(())
    }

    pub fn change_summary(&self) -> Result<String> {
        let Some(snapshot) = &self.snapshot else {
            return Ok("Sin lectura previa: se escribirían todas las tablas.".into());
        };
        let order = self.command_order()?;
        if order.is_empty() {
            return Ok("No hay cambios respecto a la última lectura.".into());
        }
        let mut parts = Vec::new();
        for cmd in order {
            let part = match cmd {
                SET_ACTUATION => format!(
                    "Actuación: {} teclas",
                    count_record_diffs(&self.payload(SET_ACTUATION)?, &snapshot.actuation, 8)
                ),
                SET_FN => format!(
                    "Capa Fn: {} teclas",
                    count_record_diffs(&self.payload(SET_FN)?, &snapshot.fn_layer, 4)
                ),
                SET_KEYS => format!(
                    "Capa normal: {} teclas",
                    count_record_diffs(&self.payload(SET_KEYS)?, &snapshot.base, 4)
                ),
                SET_GLOBALS => "Sistema".to_string(),
                SET_DKS => "DKS".to_string(),
                frame::SET_MACRO => "Macros".to_string(),
                SET_LIGHT => "Luz".to_string(),
                SET_COLOR => "Color por tecla".to_string(),
                _ => continue,
            };
            parts.push(part);
        }
        Ok(parts.join("; "))
    }

    pub fn actuation_bytes(&self) -> Result<[u8; actuation::TABLE]> {
        self.validate()?;
        Ok(actuation::encode_table(&self.actuation))
    }

    pub fn apply_actuation_bytes(
        &mut self,
        bytes: &[u8],
        overlay_ultra: &[bool; actuation::RECORDS],
    ) -> Result<()> {
        let records = actuation::decode_table(bytes)?;
        for (index, record) in records.into_iter().enumerate() {
            let choice = if overlay_ultra[index] && record.axis_type == 2 {
                SwitchChoice::JadeUltra
            } else {
                SwitchChoice::from_device_byte(record.axis_type)
            };
            self.choice[index] = choice;
            self.actuation[index] = record;
        }
        Ok(())
    }

    pub fn apply_info(&mut self, payload: &[u8]) -> Result<()> {
        let info = info::parse(payload)?;
        self.default_axis = switches::default_axis_for_model(info.axial_model);
        self.info = Some(info);
        Ok(())
    }

    fn slot_mut(&mut self, index: usize) -> Result<&mut Record> {
        self.actuation
            .get_mut(index)
            .ok_or(Error::Protocol("índice de tecla fuera de la tabla"))
    }
}

fn count_record_diffs(now: &[u8], before: &[u8], size: usize) -> usize {
    let slots = layout_indexes();
    slots
        .into_iter()
        .filter(|index| {
            let start = index * size;
            let end = start + size;
            now.get(start..end) != before.get(start..end)
        })
        .count()
}

fn layout_indexes() -> Vec<usize> {
    crate::layout::keys()
        .into_iter()
        .map(|key| key.index as usize)
        .collect()
}

fn record_press(board: &Board, index: usize) -> Result<u16> {
    board
        .actuation
        .get(index)
        .map(|record| record.rt_press)
        .ok_or(Error::Protocol("índice de tecla fuera de la tabla"))
}

pub fn tables_to_read() -> &'static [(u8, usize)] {
    &[
        (GET_INFO, info::TABLE),
        (GET_GLOBALS, crate::globals::TABLE),
        (GET_KEYS, keys::BASE_READ),
        (GET_FN, keys::FN_BYTES),
        (GET_ACTUATION, actuation::TABLE),
        (GET_DKS, dks::TABLE),
        (GET_LIGHT, lighting::BLOCK),
        (GET_COLOR, lighting::COLOR_TABLE),
    ]
}

pub fn load_read(board: &mut Board, cmd: u8, bytes: &[u8]) -> Result<()> {
    match cmd {
        GET_INFO => board.apply_info(bytes)?,
        GET_GLOBALS => board.globals = Globals::from_bytes(bytes)?,
        GET_KEYS => board.base = keys::decode_table(bytes)?,
        GET_FN => board.fn_layer = keys::decode_table(bytes)?,
        GET_ACTUATION => {
            let keep = [false; actuation::RECORDS];
            board.apply_actuation_bytes(bytes, &keep)?;
        }
        GET_DKS => board.dks = dks::decode_table(bytes)?,
        GET_LIGHT => board.lighting = Lighting::decode(bytes)?,
        GET_COLOR => board.colors = lighting::decode_colors(bytes)?,
        _ => return Err(Error::Protocol("tabla desconocida")),
    }
    Ok(())
}

pub fn macro_image(board: &Board) -> Result<Vec<u8>> {
    macro_store::encode_image(&board.macros)
}
