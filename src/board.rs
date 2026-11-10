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
use crate::switches::SwitchChoice;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    Empty,
    Device,
    File,
}

#[derive(Debug, Clone)]
pub struct Board {
    pub origin: Origin,
    pub info: Option<DeviceInfo>,
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
}

impl Default for Board {
    fn default() -> Self {
        Self {
            origin: Origin::Empty,
            info: None,
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
        }
    }
}

impl Board {
    pub fn ready(&self) -> bool {
        self.origin != Origin::Empty
    }

    /// Cambia el eje y, si el punto actual no cabe, lo baja al tope nuevo.
    pub fn assign_switch(&mut self, index: usize, choice: SwitchChoice) -> Result<()> {
        let max = choice.max_cmm()?;
        let record = self.slot_mut(index)?;
        if record.trip > max {
            record.trip = max;
        }
        if record.rt_press > max {
            record.rt_press = max.max(crate::switches::RT_MIN);
        }
        if record.rt_release > max {
            record.rt_release = max.max(crate::switches::RT_MIN);
        }
        record.axis_type = choice.axis_byte();
        self.choice[index] = choice;
        Ok(())
    }

    pub fn set_switch(&mut self, index: usize, choice: SwitchChoice) -> Result<()> {
        let record = self.slot_mut(index)?;
        if record.trip > choice.max_cmm()?
            || record.rt_press > choice.max_cmm()?
            || record.rt_release > choice.max_cmm()?
        {
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
        let record = self.slot_mut(index)?;
        let rampage = record.flags & actuation::FLAG_RAMPAGE;
        let axis_before = record.axis_type;
        actuation::set_trip(record, choice, trip)?;
        if choice.axis_byte() == axis_before {
            record.axis_type = axis_before;
        }
        record.flags = (record.flags & !actuation::FLAG_RAMPAGE) | rampage;
        Ok(())
    }

    pub fn set_rt_press(&mut self, index: usize, value: u16) -> Result<()> {
        let choice = self.choice[index];
        let record = self.slot_mut(index)?;
        let rampage = record.flags & actuation::FLAG_RAMPAGE;
        actuation::set_rt_press(record, choice, value)?;
        record.flags |= rampage;
        Ok(())
    }

    pub fn set_rt_release(&mut self, index: usize, value: u16) -> Result<()> {
        let choice = self.choice[index];
        let press = record_press(self, index)?;
        let record = self.slot_mut(index)?;
        let rampage = record.flags & actuation::FLAG_RAMPAGE;
        actuation::set_rt_release(record, choice, value)?;
        record.rt_press = press;
        record.flags |= rampage;
        Ok(())
    }

    pub fn set_rt_enabled(&mut self, index: usize, enabled: bool) -> Result<()> {
        let record = self.slot_mut(index)?;
        actuation::set_rt_enabled(record, enabled);
        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        for (index, record) in self.actuation.iter().enumerate() {
            actuation::validate_record(self.choice[index], record)?;
        }
        for slot in 0..dks::SLOTS {
            if self.dks[slot] != Slot::default() {
                dks::check_slot(slot as u8)?;
            }
        }
        Ok(())
    }

    pub fn command_order(&self) -> Vec<u8> {
        let mut order = vec![
            SET_GLOBALS,
            SET_ACTUATION,
            SET_DKS,
            frame::SET_MACRO,
            SET_KEYS,
            SET_FN,
            SET_LIGHT,
        ];
        if self.lighting.mode == 20 {
            order.push(SET_COLOR);
        }
        order
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
        self.info = Some(info::parse(payload)?);
        Ok(())
    }

    fn slot_mut(&mut self, index: usize) -> Result<&mut Record> {
        self.actuation
            .get_mut(index)
            .ok_or(Error::Protocol("índice de tecla fuera de la tabla"))
    }
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
