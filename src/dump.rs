use crate::actuation;
use crate::backup;
use crate::binding;
use crate::board::Board;
use crate::endpoint::Endpoint;
use crate::error::Result;
use crate::hid::Device;
use crate::layout;
use crate::switch_map::SwitchMap;
use crate::switches;
use crate::sync;

/// Lee el teclado, guarda un respaldo y describe lo que hay. No escribe.
pub fn run() -> Result<String> {
    let device = Device::open()?;
    let mut endpoint = Endpoint::new(device);
    let mut board = sync::download(&mut endpoint)?;
    let mut map_note = String::new();
    match SwitchMap::load() {
        Ok(map) => map.overlay(&mut board),
        Err(error) => {
            map_note = format!("\nMapa Jade Ultra no leído: {error}\n");
        }
    }
    let path = backup::save(&board)?;
    let mut text = report(&board);
    text.push_str(&map_note);
    text.push_str(&format!("\nRespaldo: {}\n", path.display()));
    Ok(text)
}

pub fn report(board: &Board) -> String {
    let mut lines = Vec::new();
    match &board.info {
        Some(info) => {
            let model = switches::axial_model(info.axial_model)
                .map(|entry| entry.name_es)
                .unwrap_or("sin coincidencia");
            lines.push(format!(
                "Firmware {} · VID 0x{:04X} PID 0x{:04X} · batería {}",
                info.firmware, info.vid, info.pid, info.battery
            ));
            lines.push(format!(
                "AxialModel {} ({model}) · eje de fábrica {}",
                info.axial_model,
                switches::axis_name(board.default_axis)
            ));
        }
        None => lines.push("Sin bloque de identidad.".into()),
    }
    lines.push(format!(
        "Enlace: {}",
        if board.dongle { "receptor" } else { "cable" }
    ));
    lines.push(String::new());
    for key in layout::keys() {
        let index = key.index as usize;
        let record = board.actuation[index];
        let flags = format!(
            "{}{}{}",
            if record.full_travel() {
                "recorrido "
            } else {
                ""
            },
            if record.bottom_opt() { "fondo " } else { "" },
            if record.rampage() { "rampage" } else { "" }
        );
        let flags = if flags.is_empty() {
            "-".to_string()
        } else {
            flags.trim().to_string()
        };
        lines.push(format!(
            "{:>3} {:<8} {:<28} act {}  RT {}/{} {}  flags {}  fn {}",
            key.index,
            key.label,
            board.choice[index].label_with(board.default_axis),
            actuation::mm_text(record.trip),
            actuation::mm_text(record.rt_press),
            actuation::mm_text(record.rt_release),
            if record.rt_enabled() { "on" } else { "off" },
            flags,
            binding::label_of(board.fn_layer[index]),
        ));
    }
    lines.join("\n")
}
