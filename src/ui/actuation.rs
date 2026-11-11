use eframe::egui::{self, RichText};

use hekit::actuation::{self, RT_DEFAULT};
use hekit::layout;
use hekit::switches::{SwitchChoice, AXES, JADE_PRO, JADE_ULTRA};

use super::{App, WARN};

pub fn tab(app: &mut App, ui: &mut egui::Ui) {
    ui.label(RichText::new("Switch").strong());
    let current = app
        .selected
        .iter()
        .next()
        .map(|index| app.board.choice[*index as usize]);
    ui.horizontal_wrapped(|ui| {
        for axis in AXES {
            let on = matches!(current, Some(SwitchChoice::Axis(id)) if id == axis.id);
            if ui.selectable_label(on, axis.name).clicked() {
                apply_choice(app, SwitchChoice::Axis(axis.id));
            }
        }
        if ui
            .selectable_label(
                matches!(current, Some(SwitchChoice::JadeUltra)),
                "Jade Ultra",
            )
            .clicked()
        {
            apply_choice(app, SwitchChoice::JadeUltra);
        }
    });
    if ui.button("Todas son Jade Ultra").clicked() {
        mark_all_ultra(app);
    }
    let Some(index) = app.selected.iter().next().copied() else {
        ui.label("Elige una tecla en el dibujo para ver su actuación. El color de arriba de cada tecla es el switch.");
        return;
    };
    let index = index as usize;
    if let Some(key) = layout::find(index as u16) {
        ui.label(RichText::new(format!("Tecla {}", key.label)).strong());
    }
    let choice = app.board.choice[index];
    ui.label(RichText::new(choice.label_with(app.board.default_axis)).strong());
    match choice {
        SwitchChoice::JadeUltra => {
            ui.label(format!(
                "{}: recorrido {} mm, {}–{} gf, flujo {}/{} GS. Se guarda como el id 2. Tope {} mm.",
                JADE_ULTRA.name,
                JADE_ULTRA.travel_mm,
                JADE_ULTRA.start_gf,
                JADE_ULTRA.bottom_gf,
                JADE_ULTRA.flux_start,
                JADE_ULTRA.flux_bottom,
                actuation::mm_text(JADE_ULTRA.firmware_cap)
            ));
        }
        SwitchChoice::Axis(2) => {
            ui.label(format!(
                "{}: recorrido real {} mm, {}–{} gf, flujo {}/{} GS. El firmware no pasa de {} mm.",
                JADE_PRO.name,
                JADE_PRO.travel_mm,
                JADE_PRO.start_gf,
                JADE_PRO.bottom_gf,
                JADE_PRO.flux_start,
                JADE_PRO.flux_bottom,
                actuation::mm_text(JADE_PRO.firmware_cap)
            ));
        }
        SwitchChoice::Default => {
            ui.label("El byte de eje es 0. hec lo trata como el switch que vino de fábrica.");
        }
        _ => {}
    }
    let mismatches = app.switch_map.mismatches(&app.board);
    if !mismatches.is_empty() {
        ui.label(RichText::new(format!(
            "{} teclas marcadas como Jade Ultra tienen otro id en el teclado. Pulsa «Todas son Jade Ultra» para escribir el id 2.",
            mismatches.len()
        )).color(WARN));
    }
    let record = app.board.actuation[index];
    let max = app.board.cap_of(choice).unwrap_or(340);
    let shown = record.display_trip().clamp(10, max);
    let mut trip = shown;
    ui.add(egui::Slider::new(&mut trip, 10..=max).text("Actuación (0.01 mm)"));
    if trip != shown {
        app.apply_selected(|board, key| board.set_trip(key, trip));
    } else if record.trip == 0 {
        ui.label("El teclado trae 0 en esta tecla. hec lo muestra como 1.20 mm y no lo cambia hasta que muevas el control.");
    }
    let mut enabled = record.rt_enabled();
    if ui.checkbox(&mut enabled, "Rapid Trigger").changed() {
        app.apply_selected(|board, key| board.set_rt_enabled(key, enabled));
    }
    if enabled {
        let mut whole = record.full_travel();
        if ui
            .checkbox(&mut whole, "Activación rápida de todo el recorrido")
            .changed()
        {
            app.apply_selected(|board, key| board.set_full_travel(key, whole));
        }
        let mut press = if record.rt_press == 0 {
            RT_DEFAULT
        } else {
            record.rt_press
        };
        let mut release = if record.rt_release == 0 {
            RT_DEFAULT
        } else {
            record.rt_release
        };
        ui.add(egui::Slider::new(&mut press, 1..=max).text("RT al presionar"));
        ui.add(egui::Slider::new(&mut release, 1..=max).text("RT al soltar"));
        if press <= 4 || release <= 4 {
            ui.label(
                RichText::new("Por debajo de 0.04 mm el driver marca la sensibilidad en rojo.")
                    .color(WARN),
            );
        }
        if press != record.rt_press {
            app.apply_selected(|board, key| board.set_rt_press(key, press));
        }
        if release != record.rt_release {
            app.apply_selected(|board, key| board.set_rt_release(key, release));
        }
    }
    let now = app.board.actuation[index];
    ui.label(format!(
        "Ahora: actuación {} mm, presión {} mm, liberación {} mm.",
        actuation::mm_text(now.display_trip()),
        actuation::mm_text(now.rt_press),
        actuation::mm_text(now.rt_release)
    ));
}

fn mark_all_ultra(app: &mut App) {
    app.switch_map.set_physical_all();
    for key in layout::keys() {
        if let Err(error) = app
            .board
            .assign_switch(key.index as usize, SwitchChoice::JadeUltra)
        {
            app.status = error.to_string();
            return;
        }
    }
    match app.switch_map.save() {
        Ok(path) => {
            app.status = format!(
                "Las 61 teclas quedan como Jade Ultra (id 2, tope 3.20 mm). Guardado en {}. Recalibra: deja activa la calibración adaptativa.",
                path.display()
            );
        }
        Err(error) => app.status = error.to_string(),
    }
}

fn apply_choice(app: &mut App, choice: SwitchChoice) {
    if app.selected.is_empty() {
        app.status = "Primero elige una tecla en el dibujo.".into();
        return;
    }
    let targets = app.selected_indexes();
    for target in targets {
        if let Err(error) = app.board.assign_switch(target, choice) {
            app.status = error.to_string();
            return;
        }
    }
    let indexes: Vec<u16> = app.selected.iter().copied().collect();
    if matches!(choice, SwitchChoice::JadeUltra) {
        app.switch_map.mark(indexes);
    } else {
        app.switch_map.unmark(indexes);
    }
    if let Err(error) = app.switch_map.save() {
        app.status = error.to_string();
        return;
    }
    app.status = "Eje aplicado. El driver oficial pide recalibrar después de un cambio de eje. Deja activa la calibración adaptativa; esta app no borra la calibración.".into();
}
