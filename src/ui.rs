use std::collections::BTreeSet;

use eframe::egui::{
    self, Align2, Color32, CornerRadius, FontId, Pos2, Rect, RichText, Sense, Stroke, Vec2,
};

use hekit::actuation::{self, FLAG_RT};
use hekit::board::{Board, Origin};
use hekit::endpoint::Endpoint;
use hekit::hid::Device;
use hekit::keys::{self, KeyRecord};
use hekit::layout;
use hekit::legend::{self, Caption, Dir, Mark};
use hekit::lighting::mode_name;
use hekit::profile;
use hekit::switches::{SwitchChoice, AXES};
use hekit::sync;

const BG: Color32 = Color32::from_rgb(28, 27, 25);
const PANEL: Color32 = Color32::from_rgb(42, 39, 35);
const KEY: Color32 = Color32::from_rgb(58, 53, 47);
const KEY_FN: Color32 = Color32::from_rgb(46, 58, 54);
const SELECTED: Color32 = Color32::from_rgb(232, 165, 75);
const TEXT: Color32 = Color32::from_rgb(244, 239, 230);
const MUTED: Color32 = Color32::from_rgb(176, 166, 150);

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
enum BindKind {
    Empty,
    Keyboard,
    Media,
    Mouse,
    Macro,
    Function,
}

pub struct App {
    board: Board,
    selected: BTreeSet<u16>,
    layer: Layer,
    tab: Tab,
    status: String,
    device: Option<Device>,
    bind_kind: BindKind,
    hid_code: u8,
    modifier: u8,
    media: u16,
    macro_slot: u8,
    fn_id: u8,
    socd_mode: u8,
    pair_other: u16,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        Self {
            board: Board::default(),
            selected: BTreeSet::new(),
            layer: Layer::Base,
            tab: Tab::Actuation,
            status: "Conecta el MINI 60 HE PRO por cable o por el receptor de 2.4 GHz.".into(),
            device: None,
            bind_kind: BindKind::Keyboard,
            hid_code: 0x29,
            modifier: 0,
            media: 0x00E9,
            macro_slot: 0,
            fn_id: 1,
            socd_mode: 3,
            pair_other: 51,
        }
    }

    fn connect(&mut self) {
        match Device::open() {
            Ok(device) => {
                self.status = format!(
                    "Conectado: {} · {}",
                    device.link.label(),
                    device.link.product
                );
                self.device = Some(device);
            }
            Err(error) => self.status = error.to_string(),
        }
    }

    fn read_device(&mut self) {
        let Some(device) = self.device.take() else {
            self.status = "Primero conecta el teclado.".into();
            return;
        };
        let mut endpoint = Endpoint::new(device);
        match sync::download(&mut endpoint) {
            Ok(board) => {
                self.board = board;
                self.status = firmware_status(&self.board);
            }
            Err(error) => self.status = error.to_string(),
        }
        self.device = Some(endpoint.io);
    }

    fn write_device(&mut self) {
        if !self.board.ready() {
            self.status = "Lee el teclado o importa un JSON antes de escribir.".into();
            return;
        }
        let Some(device) = self.device.take() else {
            self.status = "Conecta el teclado para escribir la configuración.".into();
            return;
        };
        let mut endpoint = Endpoint::new(device);
        match sync::upload(&mut endpoint, &self.board) {
            Ok(()) => {
                self.status = "Configuración escrita y releída. Coincide con el teclado.".into()
            }
            Err(error) => self.status = error.to_string(),
        }
        self.device = Some(endpoint.io);
    }

    fn export_json(&mut self) {
        let text = match profile::to_pretty(&self.board) {
            Ok(text) => text,
            Err(error) => {
                self.status = error.to_string();
                return;
            }
        };
        if let Some(path) = rfd::FileDialog::new()
            .set_file_name("mini60-config.json")
            .add_filter("JSON", &["json"])
            .save_file()
        {
            match std::fs::write(&path, text) {
                Ok(()) => self.status = format!("Perfil guardado en {}", path.display()),
                Err(error) => self.status = error.to_string(),
            }
        }
    }

    fn import_json(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("JSON", &["json"])
            .pick_file()
        else {
            return;
        };
        let text = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) => {
                self.status = error.to_string();
                return;
            }
        };
        match profile::parse_document(&text) {
            Ok(board) => {
                self.board = board;
                self.status = format!(
                    "Perfil cargado desde {}. Revísalo y escríbelo al teclado cuando quieras.",
                    path.display()
                );
            }
            Err(error) => self.status = error.to_string(),
        }
    }

    fn selected_indexes(&self) -> Vec<usize> {
        self.selected.iter().map(|index| *index as usize).collect()
    }

    fn paint_keyboard(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.selectable_value(&mut self.layer, Layer::Base, "Capa normal");
            ui.selectable_value(&mut self.layer, Layer::Fn, "Al mantener Fn");
            if ui.button("Seleccionar todas").clicked() {
                self.selected = layout::keys().into_iter().map(|key| key.index).collect();
            }
            if ui.button("Quitar selección").clicked() {
                self.selected.clear();
            }
        });
        let available = ui.available_size();
        if available.x < 1.0 || available.y < 1.0 {
            return;
        }
        let (canvas, _) = ui.allocate_exact_size(available, Sense::hover());
        let place = layout::place(canvas.width(), canvas.height());
        let origin = canvas.min + Vec2::new(place.origin_x, place.origin_y);
        let plate = Rect::from_min_size(origin, Vec2::new(place.board_w, place.board_h))
            .expand(place.unit * 0.05)
            .intersect(canvas);
        let plate_radius = radius_px(place.unit * 0.16, 8, 18);
        ui.painter()
            .rect_filled(plate, CornerRadius::same(plate_radius), PANEL);

        let key_radius = radius_px(place.unit * 0.12, 4, 12);
        let stroke_w = (place.unit * 0.04).clamp(1.5, 3.0);
        let keys = layout::keys();
        let fn_layer = self.layer == Layer::Fn;
        let show_strip = self.tab == Tab::Actuation;
        let show_travel = show_strip && !fn_layer;
        let shift = ui.input(|input| input.modifiers.shift);
        let table = if fn_layer {
            &self.board.fn_layer
        } else {
            &self.board.base
        };
        for key in &keys {
            let drawn = layout::key_box(place, key);
            let key_rect = Rect::from_min_size(
                canvas.min + Vec2::new(drawn.x, drawn.y),
                Vec2::new(drawn.w, drawn.h),
            );
            let response = ui.interact(key_rect, ui.id().with(key.index), Sense::click());
            if response.clicked() {
                if shift {
                    if !self.selected.insert(key.index) {
                        self.selected.remove(&key.index);
                    }
                } else {
                    self.selected.clear();
                    self.selected.insert(key.index);
                }
            }
            let selected = self.selected.contains(&key.index);
            let fill = if selected {
                SELECTED
            } else if fn_layer {
                KEY_FN
            } else {
                KEY
            };
            ui.painter()
                .rect_filled(key_rect, CornerRadius::same(key_radius), fill);
            let mut body = key_rect;
            if show_strip {
                let strip_h = (place.unit * 0.08).clamp(3.0, 7.0);
                let strip = Rect::from_min_size(key_rect.min, Vec2::new(key_rect.width(), strip_h));
                let index = key.index as usize;
                if index < self.board.choice.len() {
                    ui.painter().rect_filled(
                        strip,
                        CornerRadius::same(radius_px(strip_h * 0.6, 2, 6)),
                        switch_tint(self.board.choice[index]),
                    );
                }
                body = Rect::from_min_max(Pos2::new(body.min.x, body.min.y + strip_h), body.max);
            }
            if selected {
                ui.painter().rect_stroke(
                    key_rect,
                    CornerRadius::same(key_radius),
                    Stroke::new(stroke_w, Color32::WHITE),
                    egui::StrokeKind::Inside,
                );
            }
            let ink = if selected {
                Color32::from_rgb(40, 28, 12)
            } else {
                TEXT
            };
            let record = table.get(key.index as usize).copied().unwrap_or_default();
            let caption = legend::caption(fn_layer, key, record);
            if show_travel {
                let band_h = (key_rect.height() * 0.22).clamp(10.0, 16.0);
                let index = key.index as usize;
                if index < self.board.actuation.len() {
                    let travel = self.board.actuation[index];
                    let line = if travel.rt_enabled() {
                        format!("{} RT", actuation::mm_text(travel.trip))
                    } else {
                        actuation::mm_text(travel.trip)
                    };
                    let band = Rect::from_min_max(
                        Pos2::new(key_rect.min.x, key_rect.max.y - band_h),
                        key_rect.max,
                    );
                    let painter = ui.painter().with_clip_rect(band);
                    paint_fitted(
                        &painter,
                        band.shrink(1.0),
                        &line,
                        (place.unit * 0.16).min(12.0),
                        ink,
                    );
                }
                body = Rect::from_min_max(body.min, Pos2::new(body.max.x, body.max.y - band_h));
            }
            let painter = ui.painter().with_clip_rect(key_rect);
            paint_caption(&painter, body, &caption, ink, MUTED);
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
                    self.connect();
                }
                if ui.button("Leer").clicked() {
                    self.read_device();
                }
                if ui.button("Escribir").clicked() {
                    self.write_device();
                }
                if ui.button("Exportar JSON").clicked() {
                    self.export_json();
                }
                if ui.button("Importar JSON").clicked() {
                    self.import_json();
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
            self.paint_keyboard(ui);
        });
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
        if self.selected.is_empty()
            && matches!(
                self.tab,
                Tab::Actuation | Tab::Fn | Tab::Keys | Tab::Advanced
            )
        {
            ui.label("Elige una tecla en el dibujo. Mantén Shift para sumar varias.");
        }
        match self.tab {
            Tab::Actuation => self.actuation_tab(ui),
            Tab::Fn => self.bind_tab(ui, true),
            Tab::Keys => self.bind_tab(ui, false),
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

    fn actuation_tab(&mut self, ui: &mut egui::Ui) {
        let Some(index) = self.selected.iter().next().copied() else {
            ui.label("La actuación y el eje se aplican a las teclas seleccionadas.");
            return;
        };
        let index = index as usize;
        let choice = self.board.choice[index];
        ui.label(RichText::new(choice.label()).strong());
        if matches!(choice, SwitchChoice::JadeUltra) {
            ui.label("Jade Ultra se guarda como el id 2 de Jade Pro. El tope de esta fila es 3.20 mm, el recorrido que publica Gateron.");
        }
        let mut picked = choice_index(choice);
        egui::ComboBox::from_label("Eje")
            .selected_text(choice.label())
            .show_ui(ui, |ui| {
                for (n, axis) in AXES.iter().enumerate() {
                    ui.selectable_value(
                        &mut picked,
                        n,
                        format!("{} · tope {:.2} mm", axis.name, axis.max_cmm as f32 / 100.0),
                    );
                }
                ui.selectable_value(&mut picked, AXES.len(), "Jade Ultra · tope 3.20 mm");
            });
        if picked != choice_index(choice) {
            let next = choice_from_index(picked);
            let targets = self.selected_indexes();
            for target in targets {
                if let Err(error) = self.board.assign_switch(target, next) {
                    self.status = error.to_string();
                }
            }
            self.status = "Eje aplicado. El driver oficial pide recalibrar después de un cambio de eje; esta app no borra la calibración.".into();
        }
        let record = self.board.actuation[index];
        let max = choice.max_cmm().unwrap_or(340);
        let mut trip = record.trip;
        ui.add(egui::Slider::new(&mut trip, 10..=max).text("Actuación (0.01 mm)"));
        if trip != record.trip {
            self.apply_selected(|board, key| board.set_trip(key, trip));
        }
        let mut enabled = record.flags & FLAG_RT != 0;
        if ui.checkbox(&mut enabled, "Rapid Trigger").changed() {
            self.apply_selected(|board, key| board.set_rt_enabled(key, enabled));
        }
        let mut press = record.rt_press;
        let mut release = record.rt_release;
        ui.add(egui::Slider::new(&mut press, 1..=max).text("RT al presionar"));
        ui.add(egui::Slider::new(&mut release, 1..=max).text("RT al soltar"));
        if press != record.rt_press {
            self.apply_selected(|board, key| board.set_rt_press(key, press));
        }
        if release != record.rt_release {
            self.apply_selected(|board, key| board.set_rt_release(key, release));
        }
        ui.label(format!(
            "Ahora: actuación {} mm, presión {} mm, liberación {} mm.",
            actuation::mm_text(self.board.actuation[index].trip),
            actuation::mm_text(self.board.actuation[index].rt_press),
            actuation::mm_text(self.board.actuation[index].rt_release)
        ));
    }

    fn bind_tab(&mut self, ui: &mut egui::Ui, fn_layer: bool) {
        ui.label(if fn_layer {
            "Esto cambia lo que hace la tecla mientras mantienes Fn. La capa normal no se toca."
        } else {
            "Esto cambia la tecla en la capa normal."
        });
        egui::ComboBox::from_label("Tipo")
            .selected_text(bind_name(self.bind_kind))
            .show_ui(ui, |ui| {
                for kind in [
                    BindKind::Empty,
                    BindKind::Keyboard,
                    BindKind::Media,
                    BindKind::Mouse,
                    BindKind::Macro,
                    BindKind::Function,
                ] {
                    ui.selectable_value(&mut self.bind_kind, kind, bind_name(kind));
                }
            });
        match self.bind_kind {
            BindKind::Empty => {}
            BindKind::Keyboard => {
                ui.add(
                    egui::DragValue::new(&mut self.hid_code)
                        .hexadecimal(2, false, true)
                        .prefix("HID 0x"),
                );
                ui.add(
                    egui::DragValue::new(&mut self.modifier)
                        .hexadecimal(2, false, true)
                        .prefix("mod 0x"),
                );
            }
            BindKind::Media => {
                ui.add(
                    egui::DragValue::new(&mut self.media)
                        .hexadecimal(4, false, true)
                        .prefix("uso 0x"),
                );
            }
            BindKind::Mouse => {
                ui.label("Botón 1 es el clic izquierdo.");
                ui.add(egui::DragValue::new(&mut self.hid_code).prefix("botón "));
            }
            BindKind::Macro => {
                ui.add(
                    egui::DragValue::new(&mut self.macro_slot)
                        .range(0..=99)
                        .prefix("ranura "),
                );
            }
            BindKind::Function => {
                ui.add(egui::DragValue::new(&mut self.fn_id).prefix("función "));
                ui.label("El byte se guarda tal cual. El driver no documenta todos los ids.");
            }
        }
        if ui.button("Aplicar a la selección").clicked() {
            let record = self.current_record();
            let targets = self.selected_indexes();
            for target in targets {
                if target >= 126 {
                    continue;
                }
                if fn_layer {
                    self.board.fn_layer[target] = record;
                } else {
                    self.board.base[target] = record;
                }
            }
            self.status = if fn_layer {
                "Capa Fn actualizada en la selección.".into()
            } else {
                "Capa normal actualizada en la selección.".into()
            };
        }
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
            match keys::clear_pair(
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
            match keys::dks_bind(0) {
                Ok(record) => self.fill_selected(record),
                Err(error) => self.status = error.to_string(),
            }
        }
        ui.label("Mod-tap y toggle guardan el byte de retardo sin convertirlo.");
        if ui
            .button("Mod-tap: toque Escape, mantener Control")
            .clicked()
        {
            self.fill_selected(keys::mod_tap(0x29, 0xE0, 0x19));
        }
        if ui.button("Toggle en la tecla seleccionada").clicked() {
            self.fill_selected(keys::toggle(self.hid_code));
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
        let mode_label = mode_name(mode).to_string();
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
        ui.add(egui::Slider::new(&mut top, 0..=50).text("Zona muerta superior"));
        ui.add(egui::Slider::new(&mut bottom, 0..=50).text("Zona muerta inferior"));
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
        ui.label("La calibración manual del driver borra la calibración guardada. Aquí no existe ese botón.");
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

    fn current_record(&self) -> KeyRecord {
        match self.bind_kind {
            BindKind::Empty => KeyRecord::default(),
            BindKind::Keyboard => keys::keyboard(self.modifier, self.hid_code),
            BindKind::Media => keys::consumer(self.media),
            BindKind::Mouse => keys::mouse(1, self.hid_code.max(1)),
            BindKind::Macro => keys::macro_bind(self.macro_slot, 0, 1),
            BindKind::Function => keys::device_function(self.fn_id as u32),
        }
    }

    fn fill_selected(&mut self, record: KeyRecord) {
        let layer = self.layer;
        let targets = self.selected_indexes();
        for target in targets {
            if target >= 126 {
                continue;
            }
            if layer == Layer::Fn {
                self.board.fn_layer[target] = record;
            } else {
                self.board.base[target] = record;
            }
        }
        self.status = "Asignación avanzada aplicada.".into();
    }

    fn write_pair(&mut self, socd: bool) {
        let Some(first) = self.selected.iter().next().copied() else {
            self.status = "Selecciona la primera tecla del par.".into();
            return;
        };
        let record = if socd {
            keys::socd(self.socd_mode, self.hid_code, 0)
        } else {
            keys::rapid_snap(self.hid_code, 0)
        };
        match keys::apply_pair(
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

fn choice_index(choice: SwitchChoice) -> usize {
    match choice {
        SwitchChoice::JadeUltra => AXES.len(),
        SwitchChoice::Axis(id) | SwitchChoice::Unlisted(id) => {
            AXES.iter().position(|axis| axis.id == id).unwrap_or(0)
        }
    }
}

fn choice_from_index(index: usize) -> SwitchChoice {
    if index == AXES.len() {
        SwitchChoice::JadeUltra
    } else {
        SwitchChoice::Axis(AXES[index].id)
    }
}

fn bind_name(kind: BindKind) -> &'static str {
    match kind {
        BindKind::Empty => "Vacío",
        BindKind::Keyboard => "Tecla",
        BindKind::Media => "Media",
        BindKind::Mouse => "Ratón",
        BindKind::Macro => "Macro",
        BindKind::Function => "Función del teclado",
    }
}

fn radius_px(value: f32, min: u8, max: u8) -> u8 {
    value.round().clamp(f32::from(min), f32::from(max)) as u8
}

fn switch_tint(choice: SwitchChoice) -> Color32 {
    match choice {
        SwitchChoice::JadeUltra => Color32::from_rgb(72, 214, 146),
        SwitchChoice::Axis(id) | SwitchChoice::Unlisted(id) => {
            const TINTS: [[u8; 3]; 8] = [
                [196, 168, 96],
                [90, 86, 80],
                [120, 196, 150],
                [180, 90, 70],
                [150, 170, 190],
                [170, 170, 160],
                [140, 180, 200],
                [190, 210, 220],
            ];
            let index = usize::from(id.saturating_sub(1)) % TINTS.len();
            Color32::from_rgb(TINTS[index][0], TINTS[index][1], TINTS[index][2])
        }
    }
}

fn paint_caption(
    painter: &egui::Painter,
    rect: Rect,
    caption: &Caption,
    ink: Color32,
    hint: Color32,
) {
    if rect.width() < 1.0 || rect.height() < 1.0 {
        return;
    }
    let mut body = rect;
    if let Some(label) = caption.hint {
        let band_h = (rect.height() * 0.28).clamp(9.0, 16.0);
        let hint_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), band_h));
        paint_fitted(painter, hint_rect.shrink(1.0), label, band_h * 0.85, hint);
        body = Rect::from_min_max(Pos2::new(rect.min.x, rect.min.y + band_h), rect.max);
    }
    match &caption.mark {
        Mark::Arrow(dir) => paint_arrow(painter, body, *dir, ink),
        Mark::Text(text) => {
            let (inner_w, inner_h) = layout::legend_inner(body.width(), body.height());
            let inner = Rect::from_center_size(
                body.center(),
                Vec2::new(inner_w.min(body.width()), inner_h.min(body.height())),
            );
            paint_fitted(painter, inner, text.as_ref(), inner_h * 0.72, ink);
        }
    }
}

fn paint_fitted(painter: &egui::Painter, rect: Rect, text: &str, max_font: f32, color: Color32) {
    if rect.width() < 1.0 || rect.height() < 1.0 || text.is_empty() {
        return;
    }
    let fitted = layout::fit_caption(text, rect.width(), rect.height());
    let mut size = fitted.font.min(max_font).max(6.0);
    for step in 0..8 {
        let galley = painter.layout(
            fitted.text.clone(),
            FontId::proportional(size),
            color,
            rect.width().max(1.0),
        );
        let size_px = galley.size();
        let fits = size_px.x <= rect.width() + 0.5 && size_px.y <= rect.height() + 0.5;
        if fits || size <= 6.0 || step == 7 {
            let placed = Align2::CENTER_CENTER.anchor_size(rect.center(), size_px);
            painter.galley(placed.min, galley, color);
            return;
        }
        size *= 0.9;
    }
}

fn paint_arrow(painter: &egui::Painter, rect: Rect, dir: Dir, color: Color32) {
    let side = rect.width().min(rect.height()) * 0.42;
    if side < 2.0 {
        return;
    }
    let center = rect.center();
    let half = side * 0.5;
    let wing = side * 0.38;
    let points = match dir {
        Dir::Up => vec![
            center + Vec2::new(0.0, -half),
            center + Vec2::new(wing, half * 0.7),
            center + Vec2::new(-wing, half * 0.7),
        ],
        Dir::Down => vec![
            center + Vec2::new(0.0, half),
            center + Vec2::new(-wing, -half * 0.7),
            center + Vec2::new(wing, -half * 0.7),
        ],
        Dir::Left => vec![
            center + Vec2::new(-half, 0.0),
            center + Vec2::new(half * 0.7, wing),
            center + Vec2::new(half * 0.7, -wing),
        ],
        Dir::Right => vec![
            center + Vec2::new(half, 0.0),
            center + Vec2::new(-half * 0.7, -wing),
            center + Vec2::new(-half * 0.7, wing),
        ],
    };
    painter.add(egui::Shape::convex_polygon(points, color, Stroke::NONE));
}

fn firmware_status(board: &Board) -> String {
    match &board.info {
        Some(info) => format!(
            "Leído. Firmware {} · VID 0x{:04X} PID 0x{:04X}",
            info.firmware, info.vid, info.pid
        ),
        None => "Leído, sin bloque de identidad.".into(),
    }
}
