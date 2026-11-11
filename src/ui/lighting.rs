use eframe::egui::{self, RichText};

use hekit::lighting::{self, COLOR_RECORDS};

use super::{App, WARN};

pub fn tab(app: &mut App, ui: &mut egui::Ui) {
    let mode = app.board.lighting.mode;
    let label = if mode <= 20 {
        lighting::mode_name(mode).to_string()
    } else {
        format!("Modo del firmware ({mode})")
    };
    egui::ComboBox::from_id_salt("light-mode")
        .selected_text(label)
        .show_ui(ui, |ui| {
            for id in 0..=20 {
                if ui
                    .selectable_label(mode == id, lighting::mode_name(id))
                    .clicked()
                {
                    app.board.lighting.mode = id;
                }
            }
        });
    if mode > 20 {
        ui.label("Este modo no está en la lista. Elegir uno de arriba lo sustituye.");
    }

    let mut brightness = app.board.lighting.brightness;
    if ui
        .add(
            egui::Slider::new(&mut brightness, 0..=5)
                .clamping(egui::SliderClamping::Edits)
                .text("Brillo"),
        )
        .changed()
    {
        app.board.lighting.brightness = brightness;
    }
    let speed_mode = app.board.lighting.mode;
    let mut speed = app.board.lighting.speed;
    if ui
        .add_enabled(
            speed_mode != 0 && speed_mode != 1,
            egui::Slider::new(&mut speed, 0..=5)
                .clamping(egui::SliderClamping::Edits)
                .text("Velocidad"),
        )
        .changed()
    {
        app.board.lighting.speed = speed;
    }
    if app.board.lighting.brightness > 5 || app.board.lighting.speed > 5 {
        ui.label(
            RichText::new("Brillo y velocidad tienen que estar entre 0 y 5 para escribir la luz.")
                .color(WARN),
        );
    }

    match app.board.lighting.color_mode {
        0 | 1 => {
            let mut random = app.board.lighting.color_mode == 1;
            if ui.checkbox(&mut random, "Color aleatorio").changed() {
                app.board.lighting.color_mode = u8::from(random);
            }
        }
        other => {
            ui.label(format!("Modo de color del firmware: {other}."));
            if ui.button("Color fijo").clicked() {
                app.board.lighting.color_mode = 0;
            }
            if ui.button("Color aleatorio").clicked() {
                app.board.lighting.color_mode = 1;
            }
        }
    }

    ui.label("Color del efecto");
    let before = app.board.lighting.color;
    let mut color = before;
    if ui.color_edit_button_srgb(&mut color).changed() && color != before {
        app.board.lighting.color = color;
    }

    let mut direction = app.board.lighting.direction;
    if ui
        .add(
            egui::DragValue::new(&mut direction)
                .range(0..=255)
                .prefix("Dirección "),
        )
        .changed()
    {
        app.board.lighting.direction = direction;
    }
    ui.label("La dirección es el byte 11. El teclado lo interpreta según el efecto.");

    ui.separator();
    ui.label("Pincel para el color por tecla");
    let brush_before = app.brush;
    let mut brush = brush_before;
    if ui.color_edit_button_srgb(&mut brush).changed() && brush != brush_before {
        app.brush = brush;
    }
    if app.board.lighting.mode == 20 {
        ui.label("El dibujo muestra el color guardado de cada tecla. Todas y Fila eligen; este botón pinta.");
        if ui.button("Pintar selección").clicked() {
            paint_selection(app);
        }
    } else {
        ui.label(
            "El color por tecla se envía solo en el modo Color por tecla. Si lo pintas y cambias de modo, se queda en la ventana.",
        );
    }
}

fn paint_selection(app: &mut App) {
    if app.selected.is_empty() {
        app.status = "Selecciona teclas, una fila o Todas.".into();
        return;
    }
    let color = app.brush;
    let mut painted = 0usize;
    for index in app.selected_indexes() {
        if index < COLOR_RECORDS {
            app.board.colors[index] = color;
            painted += 1;
        }
    }
    app.status =
        format!("Pincel aplicado a {painted} teclas. Se escribe con el modo Color por tecla.");
}
