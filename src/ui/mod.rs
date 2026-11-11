mod actuation;
mod bindings;
mod keyboard;
mod sync;

use std::collections::BTreeSet;

use eframe::egui::{self, Color32, RichText};

use hekit::board::{Board, Origin};
use hekit::hid::Device;
use hekit::keys::KeyRecord;
use hekit::switch_map::SwitchMap;

const BG: Color32 = Color32::from_rgb(28, 27, 25);
const PANEL: Color32 = Color32::from_rgb(42, 39, 35);
const KEY: Color32 = Color32::from_rgb(58, 53, 47);
const KEY_FN: Color32 = Color32::from_rgb(46, 58, 54);
const SELECTED: Color32 = Color32::from_rgb(232, 165, 75);
const TEXT: Color32 = Color32::from_rgb(244, 239, 230);
const MUTED: Color32 = Color32::from_rgb(176, 166, 150);
const WARN: Color32 = Color32::from_rgb(214, 92, 78);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Layer {
    Base,
    Fn,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Actuation,
    Fn,
    Keys,
    Advanced,
    Macros,
    Light,
    System,
}

pub struct App {
    board: Board,
    selected: BTreeSet<u16>,
    layer: Layer,
    tab: Tab,
    status: String,
    device: Option<Device>,
    socd_mode: u8,
    pair_other: u16,
    hid_code: u8,
    search: String,
    capture_key: bool,
    confirm_write: bool,
    macro_slot: u8,
    switch_map: SwitchMap,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        let switch_map = SwitchMap::load().unwrap_or_default();
        Self {
            board: Board::default(),
            selected: BTreeSet::new(),
            layer: Layer::Base,
            tab: Tab::Actuation,
            status: "Conecta el MINI 60 HE PRO por cable o por el receptor de 2.4 GHz.".into(),
            device: None,
            socd_mode: 3,
            pair_other: 51,
            hid_code: 0x04,
            search: String::new(),
            capture_key: false,
            confirm_write: false,
            macro_slot: 0,
            switch_map,
        }
    }

    fn selected_indexes(&self) -> Vec<usize> {
        self.selected.iter().map(|index| *index as usize).collect()
    }

    fn apply_selected(&mut self, mut op: impl FnMut(&mut Board, usize) -> hekit::Result<()>) {
        let targets = self.selected_indexes();
        for target in targets {
            if let Err(error) = op(&mut self.board, target) {
                self.status = error.to_string();
                return;
            }
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("bar").show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("hekit").strong().color(SELECTED));
                ui.label(RichText::new("MINI 60 HE PRO").color(TEXT));
                if ui.button("Conectar").clicked() {
                    sync::connect(self);
                }
                if ui.button("Leer").clicked() {
                    sync::read_device(self);
                }
                if ui.button("Escribir").clicked() {
                    self.confirm_write = true;
                }
                if ui.button("Exportar JSON").clicked() {
                    sync::export_json(self);
                }
                if ui.button("Importar JSON").clicked() {
                    sync::import_json(self);
                }
                if ui.button("Restaurar respaldo").clicked() {
                    sync::restore_backup(self);
                }
            });
            ui.label(RichText::new(&self.status).color(MUTED).size(13.0));
        });
        egui::Panel::right("controls")
            .resizable(true)
            .default_size(380.0)
            .min_size(300.0)
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| self.side_panel(ui));
            });
        egui::CentralPanel::default().show(ui, |ui| {
            ui.visuals_mut().panel_fill = BG;
            keyboard::paint(self, ui);
        });
        sync::confirm(self, ui.ctx());
    }
}

impl App {
    fn side_panel(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            for (tab, label) in [
                (Tab::Actuation, "Actuación"),
                (Tab::Fn, "Capa Fn"),
                (Tab::Keys, "Teclas"),
                (Tab::Advanced, "Avanzadas"),
                (Tab::Macros, "Macros"),
                (Tab::Light, "Iluminación"),
                (Tab::System, "Sistema"),
            ] {
                ui.selectable_value(&mut self.tab, tab, label);
            }
        });
        ui.separator();
        ui.label("El dibujo está a la izquierda. Shift suma teclas.");
        match self.tab {
            Tab::Actuation => actuation::tab(self, ui),
            Tab::Fn => bindings::tab(self, ui, true),
            Tab::Keys => bindings::tab(self, ui, false),
            Tab::Advanced => self.advanced_tab(ui),
            Tab::Macros => self.macro_tab(ui),
            Tab::Light => self.light_tab(ui),
            Tab::System => self.system_tab(ui),
        }
        ui.add_space(8.0);
        let origin = match self.board.origin {
            Origin::Empty => "Borrador local, todavía no leído del teclado",
            Origin::Device => "Leído del teclado",
            Origin::File => "Cargado desde un archivo",
        };
        ui.label(RichText::new(origin).small().color(MUTED));
    }

    fn advanced_tab(&mut self, ui: &mut egui::Ui) {
        ui.label("SOCD y rapid snap se escriben en las dos teclas. Un solo lado se rechaza.");
        ui.add(egui::DragValue::new(&mut self.pair_other).prefix("otra tecla, índice "));
        ui.add(egui::Slider::new(&mut self.socd_mode, 1..=4).text("modo SOCD (1, 2, 3 o 4)"));
        if ui.button("Crear par SOCD").clicked() {
            self.write_pair(true);
        }
        if ui.button("Crear rapid snap").clicked() {
            self.write_pair(false);
        }
        if ui.button("Borrar el par").clicked() {
            let Some(first) = self.selected.iter().next().copied() else {
                self.status = "Selecciona una tecla del par.".into();
                return;
            };
            match hekit::keys::clear_pair(
                &mut self.board.base,
                first as usize,
                self.pair_other as usize,
            ) {
                Ok(()) => self.status = "Par borrado en las dos teclas.".into(),
                Err(error) => self.status = error.to_string(),
            }
        }
        ui.separator();
        ui.label("DKS usa décimas de milímetro: 1.6 mm se guarda como 16, no como 160.");
        if ui.button("Asignar DKS en la ranura 0").clicked() {
            match hekit::keys::dks_bind(0) {
                Ok(record) => self.fill_selected(record),
                Err(error) => self.status = error.to_string(),
            }
        }
        ui.label("Mod-tap y toggle guardan el byte de retardo sin convertirlo.");
        if ui
            .button("Mod-tap: toque Escape, mantener Control")
            .clicked()
        {
            self.fill_selected(hekit::keys::mod_tap(0x29, 0xE0, 0x19));
        }
        if ui.button("Toggle en la tecla seleccionada").clicked() {
            self.fill_selected(hekit::keys::toggle(self.hid_code));
        }
    }

    fn macro_tab(&mut self, ui: &mut egui::Ui) {
        ui.label("Una macro de ejemplo: A abajo, espera, A arriba. El retardo máximo es 65535 ms.");
        if ui.button("Añadir macro de la tecla A").clicked() {
            self.board.macros.push(vec![
                hekit::macro_store::Event {
                    delay_ms: 10,
                    hid: 0x04,
                    down: true,
                },
                hekit::macro_store::Event {
                    delay_ms: 10,
                    hid: 0x04,
                    down: false,
                },
            ]);
            self.board.macros_edited = true;
            self.status = format!(
                "Macro {} lista. Enlázala desde Capa Fn o Teclas.",
                self.board.macros.len() - 1
            );
        }
        ui.label(format!(
            "{} macros editadas en este perfil.",
            self.board.macros.len()
        ));
        if !self.board.raw_macros.is_empty() && !self.board.macros_edited {
            ui.label("Hay macros leídas del teclado. Se conservan al escribir mientras no pulses el botón de arriba.");
        }
    }

    fn light_tab(&mut self, ui: &mut egui::Ui) {
        let mut mode = self.board.lighting.mode;
        let mode_label = hekit::lighting::mode_name(mode).to_string();
        ui.add(egui::Slider::new(&mut mode, 0..=20).text(mode_label));
        if mode != self.board.lighting.mode {
            self.board.lighting.mode = mode;
        }
        let mut brightness = self.board.lighting.brightness;
        let mut speed = self.board.lighting.speed;
        ui.add(egui::Slider::new(&mut brightness, 0..=5).text("Brillo"));
        ui.add(egui::Slider::new(&mut speed, 0..=5).text("Velocidad"));
        self.board.lighting.brightness = brightness;
        self.board.lighting.speed = speed;
        let mut random = self.board.lighting.color_mode == 1;
        if ui.checkbox(&mut random, "Color aleatorio").changed() {
            self.board.lighting.color_mode = u8::from(random);
        }
        let mut color = self.board.lighting.color;
        ui.color_edit_button_srgb(&mut color);
        self.board.lighting.color = color;
        if self.board.lighting.mode == 20 {
            ui.label("El modo 20 usa el color guardado de cada tecla.");
            if let Some(index) = self.selected.iter().next().copied() {
                if (index as usize) < self.board.colors.len() {
                    ui.color_edit_button_srgb(&mut self.board.colors[index as usize]);
                }
            }
        }
        if let Err(error) = self.board.lighting.encode() {
            self.status = error.to_string();
        }
    }

    fn system_tab(&mut self, ui: &mut egui::Ui) {
        let mut rate = self.board.globals.report_rate();
        let mut sleep = self.board.globals.sleep_minutes();
        let mut top = self.board.globals.top_dead_cmm();
        let mut bottom = self.board.globals.bottom_dead_cmm();
        ui.add(egui::Slider::new(&mut rate, 0..=6).text("Tasa 0=125 Hz … 6=8 kHz"));
        ui.add(egui::Slider::new(&mut sleep, 0..=30).text("Suspensión (minutos)"));
        ui.add(egui::Slider::new(&mut top, 0..=50).text("Zona muerta superior (0.00–0.50 mm)"));
        ui.add(egui::Slider::new(&mut bottom, 0..=50).text("Zona muerta inferior (0.00–0.50 mm)"));
        let _ = self.board.globals.set_report_rate(rate);
        let _ = self.board.globals.set_sleep_minutes(sleep);
        let _ = self.board.globals.set_top_dead(top);
        let _ = self.board.globals.set_bottom_dead(bottom);
        let mut stability = self.board.globals.stability();
        let mut adaptive = self.board.globals.auto_calibration();
        if ui.checkbox(&mut stability, "Estabilidad").changed() {
            self.board.globals.set_stability(stability);
        }
        if ui
            .checkbox(&mut adaptive, "Calibración adaptativa")
            .changed()
        {
            self.board.globals.set_auto_calibration(adaptive);
        }
        ui.label("Cambiar de eje pide recalibrar en el driver oficial. Mientras hekit no tiene ese asistente, deja activa la calibración adaptativa. La calibración que borra la guardada no está en esta app.");
    }

    fn fill_selected(&mut self, record: KeyRecord) {
        let layer = self.layer;
        let targets = self.selected_indexes();
        for target in targets {
            if target >= 126 {
                continue;
            }
            if layer == Layer::Fn && hekit::binding::fn_locked(target as u16) {
                self.status = "F1 a F12 de la capa Fn son fijas.".into();
                continue;
            }
            if layer == Layer::Fn {
                self.board.fn_layer[target] = record;
            } else {
                self.board.base[target] = record;
            }
        }
    }

    fn write_pair(&mut self, socd: bool) {
        let Some(first) = self.selected.iter().next().copied() else {
            self.status = "Selecciona la primera tecla del par.".into();
            return;
        };
        let record = if socd {
            hekit::keys::socd(self.socd_mode, self.hid_code, 0)
        } else {
            hekit::keys::rapid_snap(self.hid_code, 0)
        };
        match hekit::keys::apply_pair(
            &mut self.board.base,
            first as usize,
            self.pair_other as usize,
            record,
        ) {
            Ok(()) => self.status = "Par escrito en las dos teclas.".into(),
            Err(error) => self.status = error.to_string(),
        }
    }
}
