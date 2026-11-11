use eframe::egui::{self, Color32, CornerRadius, Sense, Stroke, Vec2};

use hekit::binding;
use hekit::layout::{self, KeyGeom};

use super::{App, Layer, Tab, KEY, KEY_FN, MUTED, PANEL, SELECTED, TEXT};

pub fn paint(app: &mut App, ui: &mut egui::Ui) {
    ui.horizontal_wrapped(|ui| {
        ui.selectable_value(&mut app.layer, Layer::Base, "Capa normal");
        ui.selectable_value(&mut app.layer, Layer::Fn, "Al mantener Fn");
        if ui.button("Todas").clicked() {
            app.selected = layout::keys().into_iter().map(|key| key.index).collect();
        }
        if ui.button("Ninguna").clicked() {
            app.selected.clear();
        }
        for row in 0..5 {
            if ui.button(format!("Fila {}", row + 1)).clicked() {
                app.selected = layout::keys()
                    .into_iter()
                    .filter(|key| key.y == row as f32)
                    .map(|key| key.index)
                    .collect();
            }
        }
        if ui.button("Mismo switch").clicked() {
            select_same_switch(app);
        }
    });
    let width = ui.available_width();
    let unit = (width / 15.4).clamp(28.0, 58.0);
    let gap = unit * 0.12;
    let keys = layout::keys();
    let height = unit * 5.0 + gap * 4.0 + 8.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    ui.painter()
        .rect_filled(rect, CornerRadius::same(12), PANEL);
    let origin = rect.min + Vec2::new(unit * 0.15, unit * 0.12);
    let shift = ui.input(|input| input.modifiers.shift);
    for key in &keys {
        let key_rect = egui::Rect::from_min_size(
            origin + Vec2::new(key.x * unit, key.y * (unit + gap)),
            Vec2::new(key.w * unit - gap, unit * 0.92),
        );
        let response = ui.interact(key_rect, ui.id().with(key.index), Sense::click());
        if response.clicked() {
            if shift {
                if !app.selected.insert(key.index) {
                    app.selected.remove(&key.index);
                }
            } else {
                app.selected.clear();
                app.selected.insert(key.index);
            }
        }
        let selected = app.selected.contains(&key.index);
        let locked = app.layer == Layer::Fn && binding::fn_locked(key.index);
        let fill = if selected {
            SELECTED
        } else if locked {
            Color32::from_rgb(38, 44, 42)
        } else if app.layer == Layer::Fn {
            KEY_FN
        } else {
            KEY
        };
        ui.painter()
            .rect_filled(key_rect, CornerRadius::same(6), fill);
        if app.tab == Tab::Actuation {
            let tint = app.board.choice[key.index as usize].tint();
            let strip = egui::Rect::from_min_size(
                key_rect.min,
                Vec2::new(key_rect.width(), (unit * 0.12).max(4.0)),
            );
            ui.painter().rect_filled(
                strip,
                CornerRadius::same(4),
                Color32::from_rgb(tint[0], tint[1], tint[2]),
            );
        }
        if selected {
            ui.painter().rect_stroke(
                key_rect,
                CornerRadius::same(6),
                Stroke::new(2.0, Color32::WHITE),
                egui::StrokeKind::Inside,
            );
        }
        let ink = if selected {
            Color32::from_rgb(40, 28, 12)
        } else if locked {
            MUTED
        } else {
            TEXT
        };
        let painter = ui.painter().with_clip_rect(key_rect);
        let caption = caption(app, key);
        let svg = svg_for(app, key);
        if let Some(name) = svg {
            if let Some(bytes) = crate::legends::bytes(name) {
                let image =
                    egui::Image::from_bytes(format!("bytes://keys/{name}.svg"), bytes.to_vec())
                        .tint(ink)
                        .fit_to_exact_size(key_rect.size() * 0.55);
                ui.put(key_rect.shrink(8.0), image);
            }
        } else {
            painter.text(
                key_rect.center(),
                egui::Align2::CENTER_CENTER,
                caption,
                egui::FontId::proportional((unit * 0.22).clamp(10.0, 16.0)),
                ink,
            );
        }
        if app.layer == Layer::Fn {
            ui.painter().text(
                key_rect.left_top() + Vec2::new(4.0, 3.0),
                egui::Align2::LEFT_TOP,
                key.label,
                egui::FontId::proportional((unit * 0.16).max(8.0)),
                MUTED,
            );
        }
        if app.tab == Tab::Actuation {
            let record = app.board.actuation[key.index as usize];
            let line = format!(
                "{} {}",
                hekit::actuation::mm_text(record.display_trip()),
                if record.rt_enabled() { "RT" } else { "" }
            );
            painter.text(
                key_rect.center_bottom() - Vec2::new(0.0, 3.0),
                egui::Align2::CENTER_BOTTOM,
                line,
                egui::FontId::proportional((unit * 0.16).max(8.0)),
                ink,
            );
        }
    }
}

fn select_same_switch(app: &mut App) {
    let Some(first) = app.selected.iter().next().copied() else {
        app.status = "Selecciona una tecla para copiar su switch.".into();
        return;
    };
    let choice = app.board.choice[first as usize];
    app.selected = layout::keys()
        .into_iter()
        .filter(|key| app.board.choice[key.index as usize] == choice)
        .map(|key| key.index)
        .collect();
}

fn caption(app: &App, key: &KeyGeom) -> String {
    if app.tab == Tab::Actuation && app.layer == Layer::Base {
        return key.label.to_string();
    }
    let table = if app.layer == Layer::Fn {
        &app.board.fn_layer
    } else {
        &app.board.base
    };
    let record = table.get(key.index as usize).copied().unwrap_or_default();
    if app.layer == Layer::Fn {
        if let Some(slot) = binding::factory_at(key.index) {
            if slot.locked || record.is_empty() {
                return slot.label.to_string();
            }
        }
    }
    if record.is_empty() {
        key.label.to_string()
    } else {
        binding::label_of(record)
    }
}

fn svg_for(app: &App, key: &KeyGeom) -> Option<&'static str> {
    if app.tab == Tab::Actuation && app.layer == Layer::Base {
        return key.svg;
    }
    let table = if app.layer == Layer::Fn {
        &app.board.fn_layer
    } else {
        &app.board.base
    };
    let record = table.get(key.index as usize).copied().unwrap_or_default();
    if app.layer == Layer::Fn && binding::fn_locked(key.index) {
        return None;
    }
    if record.is_empty() && app.layer == Layer::Base {
        return key.svg;
    }
    binding::svg_name(record)
}
