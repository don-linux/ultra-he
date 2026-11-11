use eframe::egui::{self, Align2, Color32, CornerRadius, FontId, Pos2, Rect, Sense, Stroke, Vec2};

use hekit::binding;
use hekit::layout;
use hekit::legend::{self, Caption, Dir, Mark};

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
    ui.painter().rect_filled(
        plate,
        CornerRadius::same(radius_px(place.unit * 0.16, 8, 18)),
        PANEL,
    );

    let key_radius = radius_px(place.unit * 0.12, 4, 12);
    let stroke_w = (place.unit * 0.04).clamp(1.5, 3.0);
    let fn_layer = app.layer == Layer::Fn;
    let show_strip = app.tab == Tab::Actuation;
    let show_travel = show_strip && !fn_layer;
    let shift = ui.input(|input| input.modifiers.shift);
    let table = if fn_layer {
        &app.board.fn_layer
    } else {
        &app.board.base
    };
    for key in &layout::keys() {
        let drawn = layout::key_box(place, key);
        let key_rect = Rect::from_min_size(
            canvas.min + Vec2::new(drawn.x, drawn.y),
            Vec2::new(drawn.w, drawn.h),
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
        let locked = fn_layer && binding::fn_locked(key.index);
        let rgb = (app.tab == Tab::Light && app.board.lighting.mode == 20)
            .then(|| app.board.colors.get(key.index as usize).copied())
            .flatten();
        let fill = if let Some(color) = rgb {
            Color32::from_rgb(color[0], color[1], color[2])
        } else if selected {
            SELECTED
        } else if locked {
            Color32::from_rgb(38, 44, 42)
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
            let index = key.index as usize;
            if let Some(choice) = app.board.choice.get(index) {
                let tint = choice.tint();
                let strip = Rect::from_min_size(key_rect.min, Vec2::new(key_rect.width(), strip_h));
                ui.painter().rect_filled(
                    strip,
                    CornerRadius::same(radius_px(strip_h * 0.6, 2, 6)),
                    Color32::from_rgb(tint[0], tint[1], tint[2]),
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
        let ink = if let Some(color) = rgb {
            ink_on(color)
        } else if selected {
            Color32::from_rgb(40, 28, 12)
        } else if locked {
            MUTED
        } else {
            TEXT
        };
        let record = table.get(key.index as usize).copied().unwrap_or_default();
        let caption = legend::caption(fn_layer, key, record);
        if show_travel {
            let band_h = (key_rect.height() * 0.22).clamp(10.0, 16.0);
            if let Some(travel) = app.board.actuation.get(key.index as usize).copied() {
                let mm = hekit::actuation::mm_text(travel.display_trip());
                let line = if travel.rt_enabled() {
                    format!("{mm} RT")
                } else {
                    mm
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

fn ink_on(color: [u8; 3]) -> Color32 {
    let luma = u32::from(color[0]) * 3 + u32::from(color[1]) * 6 + u32::from(color[2]);
    if luma > 1_270 {
        Color32::from_rgb(28, 27, 25)
    } else {
        TEXT
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

fn radius_px(value: f32, min: u8, max: u8) -> u8 {
    value.round().clamp(f32::from(min), f32::from(max)) as u8
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
