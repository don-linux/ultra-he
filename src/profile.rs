use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::actuation::{self, Record};
use crate::board::{Board, Origin};
use crate::dks::{self, Slot};
use crate::error::{Error, Result};
use crate::globals::Globals;
use crate::info::DeviceInfo;
use crate::keys::{self, KeyRecord};
use crate::lighting::{self, Lighting};
use crate::link::PID_WIRED;
use crate::macro_store::Event;
use crate::switches::SwitchChoice;

pub const FORMAT: &str = "mini60-config";
pub const VERSION: u32 = 2;
pub const HEC_PRODUCT: u64 = 32930;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Profile {
    pub format: String,
    pub version: u32,
    pub device: DeviceJson,
    pub system: SystemJson,
    #[serde(default)]
    pub lighting: LightJson,
    #[serde(default)]
    pub per_key_color: BTreeMap<u16, [u8; 3]>,
    #[serde(default)]
    pub actuation: BTreeMap<u16, ActuationJson>,
    #[serde(default)]
    pub keys: BTreeMap<u16, KeyJson>,
    #[serde(default)]
    pub fn_layer: BTreeMap<u16, KeyJson>,
    #[serde(default)]
    pub socd: Vec<PairJson>,
    #[serde(default)]
    pub rapid_snap: Vec<PairJson>,
    #[serde(default)]
    pub dks: Vec<DksJson>,
    #[serde(default)]
    pub macros: Vec<MacroJson>,
    #[serde(default)]
    pub vendor_packs: Option<VendorPacks>,
    /// Tablas crudas. Si vienen, ganan sobre los campos editados.
    #[serde(default)]
    pub raw: Option<RawTables>,
    /// Vista para leer el archivo sin decodificar bytes.
    #[serde(default)]
    pub view: Vec<ViewKey>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeviceJson {
    pub vid: String,
    pub pid: String,
    #[serde(default)]
    pub firmware: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemJson {
    pub report_rate: u8,
    pub sleep_minutes: u8,
    pub top_dead_cmm: u8,
    pub bottom_dead_cmm: u8,
    pub stability: bool,
    pub auto_calibration: bool,
    #[serde(default)]
    pub raw: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LightJson {
    pub mode: u8,
    pub color: [u8; 3],
    pub color_mode: u8,
    pub brightness: u8,
    pub speed: u8,
    pub direction: u8,
}

impl Default for LightJson {
    fn default() -> Self {
        let light = Lighting::default();
        Self {
            mode: light.mode,
            color: light.color,
            color_mode: light.color_mode,
            brightness: light.brightness,
            speed: light.speed,
            direction: light.direction,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActuationJson {
    pub switch_profile: String,
    pub rt: bool,
    pub actuation_cmm: u16,
    pub rt_press_cmm: u16,
    pub rt_release_cmm: u16,
    #[serde(default)]
    pub flags: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KeyJson {
    pub page: u8,
    pub b1: u8,
    pub b2: u8,
    pub b3: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PairJson {
    pub a: u16,
    pub b: u16,
    #[serde(default)]
    pub mode: u8,
    pub hid_a: u8,
    pub hid_b: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DksJson {
    pub slot: u8,
    pub press: u8,
    pub bottom: u8,
    pub release: u8,
    pub reset: u8,
    pub actions: [u8; 4],
    pub masks: [u8; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MacroJson {
    pub events: Vec<EventJson>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventJson {
    pub delay_ms: u16,
    pub hid: u8,
    pub down: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RawTables {
    #[serde(default)]
    pub globals: Vec<u8>,
    #[serde(default)]
    pub actuation: Vec<u8>,
    #[serde(default)]
    pub base: Vec<u8>,
    #[serde(default)]
    pub fn_layer: Vec<u8>,
    #[serde(default)]
    pub dks: Vec<u8>,
    #[serde(default)]
    pub macros: Vec<u8>,
    #[serde(default)]
    pub light: Vec<u8>,
    #[serde(default)]
    pub colors: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ViewKey {
    pub index: u16,
    pub name: String,
    pub switch: String,
    pub actuation_mm: String,
    pub rapid_trigger: bool,
    pub press_mm: String,
    pub release_mm: String,
    pub base: String,
    pub fn_layer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct VendorPacks {
    #[serde(default)]
    pub all_fn_pack: Vec<u8>,
    #[serde(default)]
    pub all_key_pack: Vec<u8>,
    #[serde(default)]
    pub rt_list: Vec<u8>,
    #[serde(default)]
    pub all_dks_pack: Vec<u8>,
    #[serde(default)]
    pub all_macro_pack: Vec<u8>,
    #[serde(default)]
    pub all_color_pack: Vec<u8>,
    #[serde(default)]
    pub all_led_pack: Vec<u8>,
}

pub fn to_profile(board: &Board) -> Result<Profile> {
    board.validate()?;
    let mut actuation = BTreeMap::new();
    for index in 0..actuation::RECORDS {
        let record = board.actuation[index];
        actuation.insert(
            index as u16,
            ActuationJson {
                switch_profile: board.choice[index].profile_key(),
                rt: record.rt_enabled(),
                actuation_cmm: record.trip,
                rt_press_cmm: record.rt_press,
                rt_release_cmm: record.rt_release,
                flags: record.flags,
            },
        );
    }
    Ok(Profile {
        format: FORMAT.into(),
        version: VERSION,
        device: DeviceJson {
            vid: format!(
                "0x{:04X}",
                board.info.as_ref().map(|info| info.vid).unwrap_or(0x0C45)
            ),
            pid: format!(
                "0x{:04X}",
                board
                    .info
                    .as_ref()
                    .map(|info| info.pid)
                    .unwrap_or(PID_WIRED)
            ),
            firmware: board
                .info
                .as_ref()
                .map(|info| info.firmware.clone())
                .unwrap_or_default(),
        },
        system: SystemJson {
            report_rate: board.globals.report_rate(),
            sleep_minutes: board.globals.sleep_minutes(),
            top_dead_cmm: board.globals.top_dead_cmm(),
            bottom_dead_cmm: board.globals.bottom_dead_cmm(),
            stability: board.globals.stability(),
            auto_calibration: board.globals.auto_calibration(),
            raw: board.globals.raw.to_vec(),
        },
        lighting: LightJson {
            mode: board.lighting.mode,
            color: board.lighting.color,
            color_mode: board.lighting.color_mode,
            brightness: board.lighting.brightness,
            speed: board.lighting.speed,
            direction: board.lighting.direction,
        },
        per_key_color: board
            .colors
            .iter()
            .enumerate()
            .map(|(i, c)| (i as u16, *c))
            .collect(),
        actuation,
        keys: key_map(&board.base),
        fn_layer: key_map(&board.fn_layer),
        socd: Vec::new(),
        rapid_snap: Vec::new(),
        dks: board
            .dks
            .iter()
            .enumerate()
            .filter(|(_, slot)| **slot != Slot::default())
            .map(|(index, slot)| DksJson {
                slot: index as u8,
                press: slot.press,
                bottom: slot.bottom,
                release: slot.release,
                reset: slot.reset,
                actions: slot.actions,
                masks: slot.masks,
            })
            .collect(),
        macros: board
            .macros
            .iter()
            .map(|events| MacroJson {
                events: events
                    .iter()
                    .map(|event| EventJson {
                        delay_ms: event.delay_ms,
                        hid: event.hid,
                        down: event.down,
                    })
                    .collect(),
            })
            .collect(),
        vendor_packs: Some(VendorPacks {
            all_fn_pack: keys::fn_write_bytes(&board.fn_layer),
            all_key_pack: keys::encode_table(&board.base).to_vec(),
            rt_list: {
                let table = actuation::encode_table(&board.actuation);
                let mut padded = table.to_vec();
                padded.resize(1024, 0);
                padded
            },
            all_dks_pack: dks::encode_table(&board.dks).to_vec(),
            all_macro_pack: crate::board::macro_image(board).unwrap_or_default(),
            all_color_pack: lighting::encode_colors(&board.colors).to_vec(),
            all_led_pack: board.lighting.encode()?.to_vec(),
        }),
        raw: Some(raw_tables(board)?),
        view: readable(board),
    })
}

fn raw_tables(board: &Board) -> Result<RawTables> {
    Ok(RawTables {
        globals: board.globals.raw.to_vec(),
        actuation: actuation::encode_table(&board.actuation).to_vec(),
        base: keys::encode_table(&board.base).to_vec(),
        fn_layer: keys::fn_write_bytes(&board.fn_layer),
        dks: dks::encode_table(&board.dks).to_vec(),
        macros: if board.macros_edited || board.origin == Origin::File {
            crate::board::macro_image(board).unwrap_or_default()
        } else {
            board.raw_macros.clone()
        },
        light: board.lighting.encode()?.to_vec(),
        colors: lighting::encode_colors(&board.colors).to_vec(),
    })
}

fn readable(board: &Board) -> Vec<ViewKey> {
    crate::layout::keys()
        .into_iter()
        .map(|key| {
            let index = key.index as usize;
            let record = board.actuation[index];
            ViewKey {
                index: key.index,
                name: key.label.to_string(),
                switch: board.choice[index].label_with(board.default_axis),
                actuation_mm: actuation::mm_text(record.display_trip()),
                rapid_trigger: record.rt_enabled(),
                press_mm: actuation::mm_text(record.rt_press),
                release_mm: actuation::mm_text(record.rt_release),
                base: crate::binding::label_of(board.base[index]),
                fn_layer: crate::binding::label_of(board.fn_layer[index]),
            }
        })
        .collect()
}

pub fn from_profile(profile: &Profile) -> Result<Board> {
    if profile.format != FORMAT {
        return Err(Error::Json(
            "el archivo no es un perfil mini60-config".into(),
        ));
    }
    if profile.version != 1 && profile.version != VERSION {
        return Err(Error::Json(format!(
            "versión {} no soportada",
            profile.version
        )));
    }
    let pid = parse_id(&profile.device.pid)?;
    if pid != PID_WIRED {
        return Err(Error::Json(format!(
            "product id 0x{pid:04X} no es el MINI 60 HE PRO"
        )));
    }
    let mut board = Board {
        origin: Origin::File,
        info: Some(DeviceInfo {
            vid: parse_id(&profile.device.vid)?,
            pid,
            firmware: profile.device.firmware.clone(),
            battery: 0,
            axial_model: 0,
        }),
        ..Board::default()
    };
    if profile.system.raw.len() == crate::globals::TABLE {
        board.globals = Globals::from_bytes(&profile.system.raw)?;
    }
    board.globals.set_report_rate(profile.system.report_rate)?;
    board
        .globals
        .set_sleep_minutes(profile.system.sleep_minutes)?;
    board.globals.set_top_dead(profile.system.top_dead_cmm)?;
    board
        .globals
        .set_bottom_dead(profile.system.bottom_dead_cmm)?;
    board.globals.set_stability(profile.system.stability);
    board
        .globals
        .set_auto_calibration(profile.system.auto_calibration);
    board.lighting = Lighting {
        mode: profile.lighting.mode,
        color: profile.lighting.color,
        color_mode: profile.lighting.color_mode,
        brightness: profile.lighting.brightness,
        speed: profile.lighting.speed,
        direction: profile.lighting.direction,
    };
    board.lighting.encode()?;
    for (index, color) in &profile.per_key_color {
        if *index < lighting::COLOR_RECORDS as u16 {
            board.colors[*index as usize] = *color;
        }
    }
    for (index, item) in &profile.actuation {
        let index = *index as usize;
        if index >= actuation::RECORDS {
            return Err(Error::Json(format!("índice de actuación {index}")));
        }
        let choice = SwitchChoice::parse_profile(&item.switch_profile)?;
        board.choice[index] = choice;
        let record = Record {
            axis_type: choice.axis_byte(),
            flags: item.flags,
            trip: item.actuation_cmm,
            rt_press: item.rt_press_cmm,
            rt_release: item.rt_release_cmm,
        };
        actuation::validate_record(choice, &record)?;
        board.actuation[index] = record;
    }
    if let Some(raw) = &profile.raw {
        apply_raw(&mut board, raw)?;
    }
    apply_key_map(&mut board.base, &profile.keys)?;
    apply_key_map(&mut board.fn_layer, &profile.fn_layer)?;
    for pair in &profile.socd {
        keys::apply_pair(
            &mut board.base,
            pair.a as usize,
            pair.b as usize,
            keys::socd(pair.mode, pair.hid_a, pair.hid_b),
        )?;
    }
    for pair in &profile.rapid_snap {
        keys::apply_pair(
            &mut board.base,
            pair.a as usize,
            pair.b as usize,
            keys::rapid_snap(pair.hid_a, pair.hid_b),
        )?;
    }
    for item in &profile.dks {
        dks::check_slot(item.slot)?;
        board.dks[item.slot as usize] = Slot {
            press: item.press,
            bottom: item.bottom,
            release: item.release,
            reset: item.reset,
            actions: item.actions,
            masks: item.masks,
        };
    }
    board.macros = profile
        .macros
        .iter()
        .map(|macro_| {
            macro_
                .events
                .iter()
                .map(|event| Event {
                    delay_ms: event.delay_ms,
                    hid: event.hid,
                    down: event.down,
                })
                .collect()
        })
        .collect();
    board.validate()?;
    Ok(board)
}

pub fn parse_document(text: &str) -> Result<Board> {
    let value: Value =
        serde_json::from_str(text).map_err(|error| Error::Json(error.to_string()))?;
    if value.get("format").and_then(Value::as_str) == Some(FORMAT) {
        let profile: Profile =
            serde_json::from_value(value).map_err(|error| Error::Json(error.to_string()))?;
        return from_profile(&profile);
    }
    from_hec(&value)
}

pub fn from_hec(value: &Value) -> Result<Board> {
    let packs = value
        .as_object()
        .ok_or_else(|| Error::Json("el perfil no es un objeto".into()))?;
    if !packs.contains_key("RTlist") && !packs.contains_key("rt_list") {
        return Err(Error::Json("el perfil hec no trae RTlist".into()));
    }
    let rt = number_array(
        packs
            .get("RTlist")
            .or_else(|| packs.get("rt_list"))
            .unwrap(),
    )?;
    if rt.len() < actuation::TABLE {
        return Err(Error::BadLength {
            what: "RTlist",
            got: rt.len(),
            expected: actuation::TABLE,
        });
    }
    let mut board = Board {
        origin: Origin::File,
        ..Board::default()
    };
    let ultra = [false; actuation::RECORDS];
    board.apply_actuation_bytes(&rt[..actuation::TABLE], &ultra)?;
    if let Some(raw) = packs.get("allFnPack").or_else(|| packs.get("all_fn_pack")) {
        let bytes = number_array(raw)?;
        if bytes.len() != keys::FN_BYTES {
            return Err(Error::BadLength {
                what: "allFnPack",
                got: bytes.len(),
                expected: keys::FN_BYTES,
            });
        }
        board.fn_layer = keys::decode_table(&bytes)?;
    }
    if let Some(raw) = packs
        .get("allKeyPack")
        .or_else(|| packs.get("all_key_pack"))
    {
        let bytes = number_array(raw)?;
        if bytes.len() < keys::BASE_READ {
            return Err(Error::BadLength {
                what: "allKeyPack",
                got: bytes.len(),
                expected: keys::BASE_READ,
            });
        }
        board.base = keys::decode_table(&bytes)?;
    }
    if let Some(raw) = packs
        .get("allDksPack")
        .or_else(|| packs.get("all_dks_pack"))
    {
        let bytes = number_array(raw)?;
        if bytes.len() >= dks::TABLE {
            board.dks = dks::decode_table(&bytes)?;
        }
    }
    if let Some(raw) = packs
        .get("allledPack")
        .or_else(|| packs.get("all_led_pack"))
    {
        let bytes = number_array(raw)?;
        if bytes.len() >= lighting::BLOCK {
            board.lighting = Lighting::decode(&bytes)?;
        }
    }
    if let Some(raw) = packs
        .get("allColorPack")
        .or_else(|| packs.get("all_color_pack"))
    {
        let bytes = number_array(raw)?;
        if bytes.len() >= lighting::COLOR_TABLE {
            board.colors = lighting::decode_colors(&bytes)?;
        }
    }
    if let Some(zone) = packs.get("zone").and_then(Value::as_object) {
        if let Some(top) = zone.get("topValue").and_then(Value::as_u64) {
            board
                .globals
                .set_top_dead(u8::try_from(top).unwrap_or(255))?;
        }
        if let Some(bottom) = zone.get("bottomValue").and_then(Value::as_u64) {
            board
                .globals
                .set_bottom_dead(u8::try_from(bottom).unwrap_or(255))?;
        }
    }
    if let Some(product) = packs.get("productId").and_then(Value::as_u64) {
        if product != HEC_PRODUCT && product != PID_WIRED as u64 {
            return Err(Error::Json(format!(
                "product id {product} no es el MINI 60 HE PRO"
            )));
        }
    }
    board.info = Some(DeviceInfo {
        vid: 0x0C45,
        pid: PID_WIRED,
        firmware: String::new(),
        battery: 0,
        axial_model: 0,
    });
    Ok(board)
}

pub fn to_pretty(board: &Board) -> Result<String> {
    let profile = to_profile(board)?;
    serde_json::to_string_pretty(&profile).map_err(|error| Error::Json(error.to_string()))
}

fn key_map(records: &[KeyRecord; keys::SLOTS]) -> BTreeMap<u16, KeyJson> {
    records
        .iter()
        .enumerate()
        .filter(|(_, record)| !record.is_empty())
        .map(|(index, record)| {
            (
                index as u16,
                KeyJson {
                    page: record.page,
                    b1: record.b1,
                    b2: record.b2,
                    b3: record.b3,
                },
            )
        })
        .collect()
}

fn apply_key_map(
    records: &mut [KeyRecord; keys::SLOTS],
    map: &BTreeMap<u16, KeyJson>,
) -> Result<()> {
    for (index, item) in map {
        let index = *index as usize;
        if index >= keys::SLOTS {
            return Err(Error::Json(format!("índice de tecla {index}")));
        }
        records[index] = KeyRecord {
            page: item.page,
            b1: item.b1,
            b2: item.b2,
            b3: item.b3,
        };
    }
    Ok(())
}

fn parse_id(text: &str) -> Result<u16> {
    let text = text.trim();
    let value = if let Some(hex) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
        u16::from_str_radix(hex, 16)
    } else {
        text.parse()
    };
    value.map_err(|_| Error::Json(format!("id {text}")))
}

fn apply_raw(board: &mut Board, raw: &RawTables) -> Result<()> {
    if raw.globals.len() == crate::globals::TABLE {
        board.globals = Globals::from_bytes(&raw.globals)?;
    }
    if raw.actuation.len() >= actuation::TABLE {
        let choices = board.choice;
        board.apply_actuation_bytes(
            &raw.actuation[..actuation::TABLE],
            &[false; actuation::RECORDS],
        )?;
        for (index, choice) in choices.into_iter().enumerate() {
            if matches!(choice, SwitchChoice::JadeUltra | SwitchChoice::Default) {
                board.choice[index] = choice;
            }
        }
    }
    if raw.base.len() >= keys::BASE_READ {
        board.base = keys::decode_table(&raw.base)?;
    }
    if raw.fn_layer.len() >= keys::FN_BYTES {
        board.fn_layer = keys::decode_table(&raw.fn_layer)?;
    }
    if raw.dks.len() >= dks::TABLE {
        board.dks = dks::decode_table(&raw.dks)?;
    }
    if raw.light.len() >= lighting::BLOCK {
        board.lighting = Lighting::decode(&raw.light)?;
    }
    if raw.colors.len() >= lighting::COLOR_TABLE {
        board.colors = lighting::decode_colors(&raw.colors)?;
    }
    if !raw.macros.is_empty() {
        board.raw_macros = raw.macros.clone();
        board.macros_edited = false;
    }
    Ok(())
}

fn byte_of(item: &Value) -> Result<u8> {
    let number = item
        .as_u64()
        .ok_or_else(|| Error::Json("byte no numérico".into()))?;
    u8::try_from(number).map_err(|_| Error::Json(format!("byte {number} no cabe")))
}

fn number_array(value: &Value) -> Result<Vec<u8>> {
    if let Some(array) = value.as_array() {
        return array.iter().map(byte_of).collect();
    }
    let Some(object) = value.as_object() else {
        return Err(Error::Json("se esperaba una lista de bytes".into()));
    };
    let mut pairs = Vec::with_capacity(object.len());
    for (key, item) in object {
        let index: usize = key
            .parse()
            .map_err(|_| Error::Json(format!("índice de pack {key}")))?;
        pairs.push((index, byte_of(item)?));
    }
    pairs.sort_by_key(|(index, _)| *index);
    let len = pairs.last().map(|(index, _)| index + 1).unwrap_or(0);
    let mut out = vec![0u8; len];
    for (index, byte) in pairs {
        out[index] = byte;
    }
    Ok(out)
}
