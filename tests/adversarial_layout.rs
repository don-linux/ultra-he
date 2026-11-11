use hekit::keys::{self, KeyRecord};
use hekit::layout::{self, KeyGeom};
use hekit::legend::{self, Caption, Dir, Mark};

fn key_named(label: &str) -> KeyGeom {
    layout::keys()
        .into_iter()
        .find(|key| key.label == label)
        .unwrap_or_else(|| panic!("falta la tecla {label}"))
}

fn text_of(caption: &Caption) -> &str {
    match &caption.mark {
        Mark::Text(text) => text.as_ref(),
        Mark::Arrow(_) => panic!("se esperaba texto"),
    }
}

fn assert_present(caption: &Caption) {
    match &caption.mark {
        Mark::Arrow(_) => {}
        Mark::Text(text) => {
            assert!(!text.is_empty());
            assert_ne!(text.as_ref(), "Tecla");
        }
    }
}

#[test]
fn base_layer_shows_every_physical_legend_including_letters() {
    let mut letters = Vec::new();
    for key in layout::keys() {
        let caption = legend::caption(false, &key, KeyRecord::default());
        assert_eq!(text_of(&caption), key.label);
        assert!(caption.hint.is_none());
        assert_present(&caption);
        let mut chars = key.label.chars();
        if let Some(letter) = chars.next() {
            if chars.next().is_none() && letter.is_ascii_uppercase() {
                letters.push(letter);
            }
        }
    }
    for letter in 'A'..='Z' {
        assert!(letters.contains(&letter), "falta {letter}");
    }
}

#[test]
fn empty_fn_layer_falls_back_to_the_physical_key() {
    for key in layout::keys() {
        let caption = legend::caption(true, &key, KeyRecord::default());
        assert_eq!(text_of(&caption), key.label);
        assert!(caption.hint.is_none());
        assert_present(&caption);
    }
}

#[test]
fn wasd_fn_arrows_are_up_left_down_right() {
    let cases = [
        ("W", 0x52_u8, Dir::Up),
        ("A", 0x50, Dir::Left),
        ("S", 0x51, Dir::Down),
        ("D", 0x4F, Dir::Right),
    ];
    for (label, hid, dir) in cases {
        let key = key_named(label);
        let caption = legend::caption(true, &key, keys::keyboard(0, hid));
        assert_eq!(caption.mark, Mark::Arrow(dir), "{label}");
        assert_eq!(caption.hint, Some(label));
    }
}

#[test]
fn w_with_the_down_hid_is_not_up() {
    let key = key_named("W");
    let caption = legend::caption(true, &key, keys::keyboard(0, 0x51));
    assert_eq!(caption.mark, Mark::Arrow(Dir::Down));
    assert_ne!(caption.mark, Mark::Arrow(Dir::Up));
}

#[test]
fn known_functions_are_named_and_unknown_bytes_use_the_physical_key() {
    let esc = key_named("Esc");
    let names = [
        (1_u32, "Restaurar fábrica"),
        (5, "Reconexión"),
        (11, "Efecto de luz"),
        (22, "Bloquear Win"),
    ];
    for (id, name) in names {
        let caption = legend::caption(true, &esc, keys::device_function(id));
        assert_eq!(text_of(&caption), name);
        assert_eq!(caption.hint, Some("Esc"));
        assert_present(&caption);
    }
    for id in 1..=22 {
        let caption = legend::caption(true, &esc, keys::device_function(id));
        assert_present(&caption);
        assert_ne!(text_of(&caption), "Esc");
    }
    let shift = key_named("Shift");
    let hid_zero = KeyRecord {
        page: 2,
        b1: 0,
        b2: 0,
        b3: 0,
    };
    let zero = legend::caption(true, &shift, hid_zero);
    assert_eq!(text_of(&zero), "Shift");
    assert!(zero.hint.is_none());

    let fn_key = key_named("Fn");
    let unknown = KeyRecord {
        page: 2,
        b1: 0,
        b2: 0xAF,
        b3: 0,
    };
    let caption = legend::caption(true, &fn_key, unknown);
    assert_eq!(text_of(&caption), "Fn");
    assert_ne!(text_of(&caption), "Tecla");
}

#[test]
fn long_fn_names_fit_inside_a_1u_key_at_the_default_window() {
    let (width, height) = layout::reference_canvas();
    let place = layout::place(width, height);
    let sample = layout::keys()
        .into_iter()
        .find(|key| (key.w - 1.0).abs() < 0.01)
        .expect("una tecla de 1u");
    let key = layout::key_box(place, &sample);
    let (inner_w, inner_h) = layout::legend_inner(key.w, key.h);
    for label in [
        "Restaurar fábrica",
        "Reconexión",
        "Bloquear Win",
        "Efecto de luz",
    ] {
        let fitted = layout::fit_caption(label, inner_w, inner_h);
        assert!(
            fitted.contained(inner_w, inner_h),
            "{label} -> {fitted:?} en {inner_w}x{inner_h}"
        );
        assert!(
            fitted.text.starts_with(&label[..4]),
            "{label} quedó en {}",
            fitted.text
        );
    }
}

#[test]
fn a_label_longer_than_the_box_is_cut_inside_and_stays_nonempty() {
    let fitted = layout::fit_caption("Restaurar fábrica", 14.0, 24.0);
    assert!(fitted.contained(14.0, 24.0), "{fitted:?}");
    assert!(!fitted.text.is_empty());
    assert!(fitted.text.contains('…'));
}

#[test]
fn place_centers_a_uniform_board_and_keeps_1u_square() {
    let avail_w = 800.0;
    let avail_h = 500.0;
    let place = layout::place(avail_w, avail_h);
    assert!((place.origin_x - (avail_w - place.board_w) * 0.5).abs() < 0.01);
    assert!((place.origin_y - (avail_h - place.board_h) * 0.5).abs() < 0.01);
    assert!((place.board_w / place.board_h - layout::COLS / layout::ROWS).abs() < 0.001);
    assert!((place.board_w - place.unit * layout::COLS).abs() < 0.01);
    assert!((place.board_h - place.unit * layout::ROWS).abs() < 0.01);
    for key in layout::keys() {
        let drawn = layout::key_box(place, &key);
        assert!(drawn.x + 0.01 >= place.origin_x);
        assert!(drawn.y + 0.01 >= place.origin_y);
        assert!(drawn.x + drawn.w <= place.origin_x + place.board_w + 0.01);
        assert!(drawn.y + drawn.h <= place.origin_y + place.board_h + 0.01);
        assert!(drawn.x + drawn.w <= avail_w + 0.01);
        assert!(drawn.y + drawn.h <= avail_h + 0.01);
        if (key.w - 1.0).abs() < 0.01 {
            assert!(
                (drawn.w - drawn.h).abs() < 0.05,
                "{} no es cuadrada: {}x{}",
                key.label,
                drawn.w,
                drawn.h
            );
        }
    }
}

#[test]
fn a_wide_short_area_is_limited_by_height_and_a_tall_one_by_width() {
    let wide = layout::place(2000.0, 200.0);
    assert!((wide.unit - 200.0 / layout::ROWS).abs() < 0.01);
    let tall = layout::place(300.0, 2000.0);
    assert!((tall.unit - 300.0 / layout::COLS).abs() < 0.01);
}

#[test]
fn doubling_the_canvas_doubles_the_unit() {
    let small = layout::place(600.0, 400.0);
    let large = layout::place(1200.0, 800.0);
    assert!((large.unit / small.unit - 2.0).abs() < 0.01);
}

#[test]
fn each_row_is_15_units_and_fn_is_the_rightmost_key() {
    let keys = layout::keys();
    assert_eq!(keys.len(), 61);
    for row in 0..5 {
        let width: f32 = keys
            .iter()
            .filter(|key| (key.y - row as f32).abs() < 0.01)
            .map(|key| key.w)
            .sum();
        assert!((width - 15.0).abs() < 0.01, "fila {row} mide {width}");
    }
    let mut right_label = "";
    let mut right_edge = -1.0;
    for key in &keys {
        if (key.y - 4.0).abs() < 0.01 && key.x + key.w > right_edge {
            right_edge = key.x + key.w;
            right_label = key.label;
        }
    }
    assert_eq!(right_label, "Fn");
}
