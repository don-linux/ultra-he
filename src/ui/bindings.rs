use eframe::egui::{self, RichText};

use hekit::binding::{self, Binding};
use hekit::keys;

use super::{App, MUTED};

pub fn tab(app: &mut App, ui: &mut egui::Ui, fn_layer: bool) {
    ui.label(if fn_layer {
        "Esto cambia lo que hace la tecla mientras mantienes Fn. La capa normal no se toca. F1 a F12 son fijas."
    } else {
        "Esto cambia la tecla en la capa normal."
    });
    let query = app.search.to_lowercase();
    ui.horizontal(|ui| {
        ui.label("Buscar");
        ui.text_edit_singleline(&mut app.search);
    });
    let mut capture = app.capture_key;
    if ui
        .checkbox(&mut capture, "Pulsa una tecla para elegirla")
        .changed()
    {
        app.capture_key = capture;
    }
    if app.capture_key {
        let mut caught = None;
        ui.input(|input| {
            for event in &input.events {
                if let egui::Event::Key {
                    key,
                    pressed: true,
                    repeat: false,
                    ..
                } = event
                {
                    caught = binding::hid_from_egui_name(&format!("{key:?}"));
                }
            }
        });
        if let Some(hid) = caught {
            app.capture_key = false;
            apply(app, fn_layer, keys::keyboard(0, hid));
            app.status = format!("Asignada {}", binding::label_of(keys::keyboard(0, hid)));
        }
    }
    ui.add(egui::Slider::new(&mut app.macro_slot, 0..=99).text("Ranura de macro"));
    if ui.button("Asignar esa macro").clicked() {
        apply(
            app,
            fn_layer,
            Binding::Macro {
                slot: app.macro_slot,
                mode: 0,
                count: 1,
            }
            .record(),
        );
    }
    egui::ScrollArea::vertical()
        .max_height(360.0)
        .show(ui, |ui| {
            let mut last_group = "";
            for item in binding::palette() {
                if !query.is_empty() && !item.label.to_lowercase().contains(&query) {
                    continue;
                }
                if item.group != last_group {
                    ui.add_space(6.0);
                    ui.label(RichText::new(item.group).strong().color(MUTED));
                    last_group = item.group;
                }
                if ui.button(item.label).clicked() {
                    apply(app, fn_layer, item.record);
                }
            }
        });
}

fn apply(app: &mut App, fn_layer: bool, record: hekit::keys::KeyRecord) {
    if app.selected.is_empty() {
        app.status = "Selecciona al menos una tecla.".into();
        return;
    }
    let targets = app.selected_indexes();
    let mut skipped = false;
    for target in targets {
        if target >= 126 {
            continue;
        }
        if fn_layer && binding::fn_locked(target as u16) {
            skipped = true;
            continue;
        }
        if fn_layer {
            app.board.fn_layer[target] = record;
        } else {
            app.board.base[target] = record;
        }
    }
    app.status = if skipped {
        "F1 a F12 de la capa Fn son fijas. El resto de la selección ya tiene la acción.".into()
    } else if fn_layer {
        "Capa Fn actualizada en la selección.".into()
    } else {
        "Capa normal actualizada en la selección.".into()
    };
}
