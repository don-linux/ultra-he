mod actuation;
mod bindings;
mod keyboard;
mod lighting;
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum IoJob {
    Idle,
    Read,
    Write,
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
    dks_slot: u8,
    brush: [u8; 3],
    switch_map: SwitchMap,
    io_job: IoJob,
    /// Frames que se pintan con el aviso antes de que el HID bloquee la ventana.
    io_delay: u8,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        let (switch_map, status) = match SwitchMap::load() {
            Ok(map) => (
                map,
                "Conecta el MINI 60 HE PRO por cable o por el receptor de 2.4 GHz.".into(),
            ),
            Err(error) => (
                SwitchMap::default(),
                format!("No se pudo leer el mapa de Jade Ultra: {error}"),
            ),
        };
        Self {
            board: Board::default(),
            selected: BTreeSet::new(),
            layer: Layer::Base,
            tab: Tab::Actuation,
            status,
            device: None,
            socd_mode: 3,
            pair_other: 51,
            hid_code: 0x04,
            search: String::new(),
            capture_key: false,
            confirm_write: false,
            macro_slot: 0,
            dks_slot: 0,
            brush: [255, 255, 255],
            switch_map,
            io_job: IoJob::Idle,
            io_delay: 0,
        }
    }

    fn flush_io(&mut self, ctx: &egui::Context) {
        if self.io_job == IoJob::Idle {
            return;
        }
        if self.io_delay > 0 {
            self.io_delay -= 1;
            ctx.request_repaint();
            return;
        }
        let job = self.io_job;
        self.io_job = IoJob::Idle;
        match job {
            IoJob::Idle => {}
            IoJob::Read => sync::read_device(self),
            IoJob::Write => sync::write_device(self),
        }
    }

    fn arm_io(&mut self, ctx: &egui::Context, job: IoJob, status: &str) {
        self.status = status.into();
        self.io_job = job;
        self.io_delay = 1;
        ctx.request_repaint();
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
        self.flush_io(ui.ctx());
        egui::Panel::top("bar").show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("hekit").strong().color(SELECTED));
                ui.label(RichText::new("MINI 60 HE PRO").color(TEXT));
                if ui.button("Conectar").clicked() {
                    sync::connect(self);
                }
                if ui.button("Leer").clicked() {
                    self.arm_io(ui.ctx(), IoJob::Read, "Leyendo el teclado…");
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
            Tab::Light => lighting::tab(self, ui),
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
            if self.layer == Layer::Fn
                && (hekit::binding::fn_locked(first) || hekit::binding::fn_locked(self.pair_other))
            {
                self.status = "F1 a F12 de la capa Fn son fijas.".into();
                return;
            }
            let table = if self.layer == Layer::Fn {
                &mut self.board.fn_layer
            } else {
                &mut self.board.base
            };
            match hekit::keys::clear_pair(table, first as usize, self.pair_other as usize) {
                Ok(()) => self.status = "Par borrado en las dos teclas.".into(),
                Err(error) => self.status = error.to_string(),
            }
        }
        ui.separator();
        ui.label("DKS usa décimas de milímetro: 1.6 mm se guarda como 16, no como 160.");
        let mut slot = self.dks_slot;
        if ui
            .add(
                egui::DragValue::new(&mut slot)
                    .range(0..=62)
                    .prefix("ranura "),
            )
            .changed()
        {
            self.dks_slot = slot;
        }
        if ui.button("Asignar esta ranura DKS").clicked() {
            match hekit::keys::dks_bind(self.dks_slot) {
                Ok(record) => self.fill_selected(record),
                Err(error) => self.status = error.to_string(),
            }
        }
        self.edit_dks(ui);
        ui.label("Mod-tap y toggle guardan el byte de retardo sin convertirlo.");
        if ui
            .button("Mod-tap: toque Escape, mantener Control")
            .clicked()
        {
            self.fill_selected(hekit::keys::mod_tap(0x29, 0xE0, 0x19));
        }
        if ui.button("Toggle en la tecla seleccionada").clicked() {
            let hid = self
                .selected
                .iter()
                .next()
                .and_then(|index| hekit::layout::find(*index))
                .map(|key| key.hid)
                .unwrap_or(self.hid_code);
            self.fill_selected(hekit::keys::toggle(hid));
        }
    }

    fn active_dks_slot(&self) -> u8 {
        if let Some(index) = self.selected.iter().next() {
            let table = if self.layer == Layer::Fn {
                &self.board.fn_layer
            } else {
                &self.board.base
            };
            let record = table[usize::from(*index)];
            if record.page == 8 && record.b1 < 63 {
                return record.b1;
            }
        }
        self.dks_slot
    }

    fn edit_dks(&mut self, ui: &mut egui::Ui) {
        let slot = usize::from(self.active_dks_slot());
        ui.label(format!("Puntos y acciones de la ranura {slot}."));
        let labels = ["presión", "fondo", "liberación", "reposo"];
        let mut points = [
            self.board.dks[slot].press,
            self.board.dks[slot].bottom,
            self.board.dks[slot].release,
            self.board.dks[slot].reset,
        ];
        let mut actions = self.board.dks[slot].actions;
        for (label, point) in labels.iter().zip(points.iter_mut()) {
            ui.add(egui::DragValue::new(point).range(0..=255).prefix(*label));
        }
        for (index, action) in actions.iter_mut().enumerate() {
            ui.add(
                egui::DragValue::new(action)
                    .range(0..=255)
                    .prefix(format!("acción {} ", index + 1)),
            );
        }
        if points
            != [
                self.board.dks[slot].press,
                self.board.dks[slot].bottom,
                self.board.dks[slot].release,
                self.board.dks[slot].reset,
            ]
            || actions != self.board.dks[slot].actions
        {
            self.board.dks[slot].press = points[0];
            self.board.dks[slot].bottom = points[1];
            self.board.dks[slot].release = points[2];
            self.board.dks[slot].reset = points[3];
            self.board.dks[slot].actions = actions;
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

    fn system_tab(&mut self, ui: &mut egui::Ui) {
        let mut rate = self.board.globals.report_rate();
        let mut sleep = self.board.globals.sleep_minutes();
        let mut top = self.board.globals.top_dead_cmm();
        let mut bottom = self.board.globals.bottom_dead_cmm();
        let rate_edit = ui
            .add(
                egui::Slider::new(&mut rate, 0..=6)
                    .clamping(egui::SliderClamping::Edits)
                    .text("Tasa 0=125 Hz … 6=8 kHz"),
            )
            .changed();
        let sleep_edit = ui
            .add(
                egui::Slider::new(&mut sleep, 0..=30)
                    .clamping(egui::SliderClamping::Edits)
                    .text("Suspensión (minutos)"),
            )
            .changed();
        let top_edit = ui
            .add(
                egui::Slider::new(&mut top, 0..=50)
                    .clamping(egui::SliderClamping::Edits)
                    .text("Zona muerta superior (0.00–0.50 mm)"),
            )
            .changed();
        let bottom_edit = ui
            .add(
                egui::Slider::new(&mut bottom, 0..=50)
                    .clamping(egui::SliderClamping::Edits)
                    .text("Zona muerta inferior (0.00–0.50 mm)"),
            )
            .changed();
        if rate_edit {
            if let Err(error) = self.board.globals.set_report_rate(rate) {
                self.status = error.to_string();
            }
        }
        if sleep_edit {
            if let Err(error) = self.board.globals.set_sleep_minutes(sleep) {
                self.status = error.to_string();
            }
        }
        if top_edit {
            if let Err(error) = self.board.globals.set_top_dead(top) {
                self.status = error.to_string();
            }
        }
        if bottom_edit {
            if let Err(error) = self.board.globals.set_bottom_dead(bottom) {
                self.status = error.to_string();
            }
        }
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
        let Some(first_key) = hekit::layout::find(first) else {
            self.status = "La primera tecla no está en el dibujo.".into();
            return;
        };
        let Some(second_key) = hekit::layout::find(self.pair_other) else {
            self.status = "La otra tecla no está en el dibujo.".into();
            return;
        };
        if self.layer == Layer::Fn
            && (hekit::binding::fn_locked(first) || hekit::binding::fn_locked(self.pair_other))
        {
            self.status = "F1 a F12 de la capa Fn son fijas.".into();
            return;
        }
        let record = if socd {
            hekit::keys::socd(self.socd_mode, first_key.hid, second_key.hid)
        } else {
            hekit::keys::rapid_snap(first_key.hid, second_key.hid)
        };
        let table = if self.layer == Layer::Fn {
            &mut self.board.fn_layer
        } else {
            &mut self.board.base
        };
        match hekit::keys::apply_pair(table, first as usize, self.pair_other as usize, record) {
            Ok(()) => self.status = "Par escrito en las dos teclas.".into(),
            Err(error) => self.status = error.to_string(),
        }
    }
}
