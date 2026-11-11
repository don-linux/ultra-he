use eframe::egui::{self, RichText};

use hekit::backup;
use hekit::board::Origin;
use hekit::endpoint::Endpoint;
use hekit::profile;
use hekit::sync;

use super::{App, SELECTED};

pub fn connect(app: &mut App) {
    match hekit::hid::Device::open() {
        Ok(device) => {
            app.status = format!(
                "Conectado: {} · {}",
                device.link.label(),
                device.link.product
            );
            app.device = Some(device);
        }
        Err(error) => app.status = error.to_string(),
    }
}

pub fn read_device(app: &mut App) {
    let Some(device) = app.device.take() else {
        app.status = "Primero conecta el teclado.".into();
        return;
    };
    let mut endpoint = Endpoint::new(device);
    match sync::download(&mut endpoint) {
        Ok(mut board) => {
            app.switch_map.overlay(&mut board);
            let mismatches = app.switch_map.mismatches(&board);
            let saved = backup::save(&board)
                .map(|path| format!(" Respaldo en {}.", path.display()))
                .unwrap_or_default();
            app.status = format!("{}{}", firmware_status(&board), saved);
            if !mismatches.is_empty() {
                app.status.push_str(&format!(
                    " {n} teclas marcadas como Jade Ultra no tienen el id 2.",
                    n = mismatches.len()
                ));
            }
            app.board = board;
        }
        Err(error) => app.status = error.to_string(),
    }
    app.device = Some(endpoint.io);
}

pub fn write_device(app: &mut App) {
    if !app.board.ready() {
        app.status = "Lee el teclado o importa un JSON antes de escribir.".into();
        return;
    }
    let Some(device) = app.device.take() else {
        app.status = "Conecta el teclado para escribir la configuración.".into();
        return;
    };
    if let Err(error) = backup::save(&app.board) {
        app.status = format!("No se pudo guardar el respaldo previo: {error}");
        app.device = Some(device);
        return;
    }
    let mut endpoint = Endpoint::new(device);
    match sync::upload(&mut endpoint, &mut app.board) {
        Ok(()) => app.status = "Configuración escrita y releída. Coincide con el teclado.".into(),
        Err(error) => app.status = error.to_string(),
    }
    app.device = Some(endpoint.io);
}

pub fn confirm(app: &mut App, ctx: &egui::Context) {
    if !app.confirm_write {
        return;
    }
    let summary = app
        .board
        .change_summary()
        .unwrap_or_else(|error| error.to_string());
    let mut accept = false;
    let mut cancel = false;
    egui::Modal::new(egui::Id::new("confirm-write")).show(ctx, |ui| {
        ui.label(
            RichText::new("Escribir al teclado")
                .strong()
                .color(SELECTED),
        );
        ui.label("Solo se envían las tablas de esta lista.");
        ui.label(&summary);
        ui.horizontal(|ui| {
            if ui.button("Escribir estos cambios").clicked() {
                accept = true;
            }
            if ui.button("Cancelar").clicked() {
                cancel = true;
            }
        });
    });
    if cancel {
        app.confirm_write = false;
    }
    if accept {
        app.confirm_write = false;
        write_device(app);
    }
}

pub fn export_json(app: &mut App) {
    let text = match profile::to_pretty(&app.board) {
        Ok(text) => text,
        Err(error) => {
            app.status = error.to_string();
            return;
        }
    };
    if let Some(path) = rfd::FileDialog::new()
        .set_file_name("mini60-config.json")
        .add_filter("JSON", &["json"])
        .save_file()
    {
        match std::fs::write(&path, text) {
            Ok(()) => app.status = format!("Perfil guardado en {}", path.display()),
            Err(error) => app.status = error.to_string(),
        }
    }
}

pub fn import_json(app: &mut App) {
    let Some(path) = rfd::FileDialog::new()
        .add_filter("JSON", &["json"])
        .pick_file()
    else {
        return;
    };
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) => {
            app.status = error.to_string();
            return;
        }
    };
    let loaded = match profile::parse_document(&text) {
        Ok(board) => board,
        Err(error) => {
            app.status = error.to_string();
            return;
        }
    };
    if app.board.origin == Origin::Device {
        let snapshot = app.board.snapshot.clone();
        let info = app.board.info.clone();
        let dongle = app.board.dongle;
        let default_axis = app.board.default_axis;
        app.board = loaded;
        app.board.snapshot = snapshot;
        app.board.origin = Origin::Device;
        app.board.dongle = dongle;
        app.board.default_axis = default_axis;
        if app.board.info.is_none() {
            app.board.info = info;
        }
        app.status = format!(
            "Perfil cargado desde {} como cambios pendientes. {}",
            path.display(),
            app.board
                .change_summary()
                .unwrap_or_else(|error| error.to_string())
        );
    } else {
        app.board = loaded;
        app.board.snapshot = None;
        app.status = format!(
            "Perfil cargado desde {}. Todavía no hay una lectura: escribir enviaría todas las tablas.",
            path.display()
        );
    }
}

pub fn restore_backup(app: &mut App) {
    let Some(path) = rfd::FileDialog::new()
        .add_filter("JSON", &["json"])
        .pick_file()
    else {
        return;
    };
    let backup = match backup::load(&path) {
        Ok(board) => board,
        Err(error) => {
            app.status = error.to_string();
            return;
        }
    };
    let Some(device) = app.device.take() else {
        app.board = backup;
        app.board.snapshot = None;
        app.status = "Respaldo cargado en la ventana. Conecta el teclado para restaurarlo.".into();
        return;
    };
    let mut endpoint = Endpoint::new(device);
    match sync::restore(&mut endpoint, &backup) {
        Ok(mut board) => {
            app.switch_map.overlay(&mut board);
            app.board = board;
            app.status = "Respaldo restaurado. Solo se reescribieron las tablas distintas.".into();
        }
        Err(error) => app.status = error.to_string(),
    }
    app.device = Some(endpoint.io);
}

fn firmware_status(board: &hekit::board::Board) -> String {
    match &board.info {
        Some(info) => {
            let axis = hekit::switches::axis_name(board.default_axis);
            format!(
                "Leído. Firmware {} · VID 0x{:04X} PID 0x{:04X} · fábrica {axis}",
                info.firmware, info.vid, info.pid
            )
        }
        None => "Leído, sin bloque de identidad.".into(),
    }
}
