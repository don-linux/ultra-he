#[derive(Debug, Clone, Copy)]
pub struct KeyGeom {
    pub index: u16,
    pub label: &'static str,
    pub svg: Option<&'static str>,
    pub hid: u8,
    pub x: f32,
    pub y: f32,
    pub w: f32,
}

fn row(y: f32, keys: &[(u16, &'static str, Option<&'static str>, u8, f32)]) -> Vec<KeyGeom> {
    let mut x = 0.0;
    let mut out = Vec::new();
    for (index, label, svg, hid, w) in keys {
        out.push(KeyGeom {
            index: *index,
            label,
            svg: *svg,
            hid: *hid,
            x,
            y,
            w: *w,
        });
        x += w;
    }
    out
}

pub fn keys() -> Vec<KeyGeom> {
    let mut all = Vec::with_capacity(61);
    all.extend(row(
        0.0,
        &[
            (0, "Esc", Some("esc"), 0x29, 1.0),
            (17, "1", Some("1"), 0x1E, 1.0),
            (18, "2", Some("2"), 0x1F, 1.0),
            (19, "3", Some("3"), 0x20, 1.0),
            (20, "4", Some("4"), 0x21, 1.0),
            (21, "5", Some("5"), 0x22, 1.0),
            (22, "6", Some("6"), 0x23, 1.0),
            (23, "7", Some("7"), 0x24, 1.0),
            (24, "8", Some("8"), 0x25, 1.0),
            (25, "9", Some("9"), 0x26, 1.0),
            (26, "0", Some("0"), 0x27, 1.0),
            (27, "-", Some("minus"), 0x2D, 1.0),
            (28, "=", Some("equals-plus"), 0x2E, 1.0),
            (92, "⌫", Some("backspace"), 0x2A, 2.0),
        ],
    ));
    all.extend(row(
        1.0,
        &[
            (32, "Tab", Some("tab"), 0x2B, 1.5),
            (33, "Q", Some("q"), 0x14, 1.0),
            (34, "W", Some("w"), 0x1A, 1.0),
            (35, "E", Some("e"), 0x08, 1.0),
            (36, "R", Some("r"), 0x15, 1.0),
            (37, "T", Some("t"), 0x17, 1.0),
            (38, "Y", Some("y"), 0x1C, 1.0),
            (39, "U", Some("u"), 0x18, 1.0),
            (40, "I", Some("i"), 0x0C, 1.0),
            (41, "O", Some("o"), 0x12, 1.0),
            (42, "P", Some("p"), 0x13, 1.0),
            (43, "[", Some("bracket-open"), 0x2F, 1.0),
            (44, "]", Some("bracket-close"), 0x30, 1.0),
            (60, "\\", Some("backslash"), 0x31, 1.5),
        ],
    ));
    all.extend(row(
        2.0,
        &[
            (48, "Caps", Some("capslock"), 0x39, 1.75),
            (49, "A", Some("a"), 0x04, 1.0),
            (50, "S", Some("s"), 0x16, 1.0),
            (51, "D", Some("d"), 0x07, 1.0),
            (52, "F", Some("f"), 0x09, 1.0),
            (53, "G", Some("g"), 0x0A, 1.0),
            (54, "H", Some("h"), 0x0B, 1.0),
            (55, "J", Some("j"), 0x0D, 1.0),
            (56, "K", Some("k"), 0x0E, 1.0),
            (57, "L", Some("l"), 0x0F, 1.0),
            (58, ";", Some("semicolon-dble"), 0x33, 1.0),
            (59, "'", Some("apostroph"), 0x34, 1.0),
            (76, "Enter", Some("enter"), 0x28, 2.25),
        ],
    ));
    all.extend(row(
        3.0,
        &[
            (64, "Shift", Some("shift"), 0xE1, 2.25),
            (65, "Z", Some("z"), 0x1D, 1.0),
            (66, "X", Some("x"), 0x1B, 1.0),
            (67, "C", Some("c"), 0x06, 1.0),
            (68, "V", Some("v"), 0x19, 1.0),
            (69, "B", Some("b"), 0x05, 1.0),
            (70, "N", Some("n"), 0x11, 1.0),
            (71, "M", Some("m"), 0x10, 1.0),
            (72, ",", Some("comma"), 0x36, 1.0),
            (73, ".", Some("period-gt"), 0x37, 1.0),
            (74, "/", Some("slash-questionmark"), 0x38, 1.0),
            (75, "Shift", Some("shift-right"), 0xE5, 2.75),
        ],
    ));
    all.extend(row(
        4.0,
        &[
            (80, "Ctrl", Some("ctrl"), 0xE0, 1.25),
            (81, "Win", Some("win"), 0xE3, 1.25),
            (82, "Alt", Some("alt"), 0xE2, 1.25),
            (83, "Espacio", Some("spacebar"), 0x2C, 6.25),
            (84, "Alt", Some("alt-right"), 0xE6, 1.25),
            (86, "Menú", Some("context-menu"), 0x65, 1.25),
            (87, "Ctrl", Some("ctrl-2"), 0xE4, 1.25),
            (85, "Fn", None, 0xFA, 1.25),
        ],
    ));
    all
}

pub fn find(index: u16) -> Option<KeyGeom> {
    keys().into_iter().find(|key| key.index == index)
}

/// Ancho del tablero en unidades de tecla. Cada fila suma esto.
pub const COLS: f32 = 15.0;
/// Filas del MINI 60.
pub const ROWS: f32 = 5.0;
/// Hueco entre teclas, como fracción de `unit`. El paso es el mismo en los dos ejes.
pub const GAP: f32 = 0.08;
/// Ancho medio de un carácter respecto al tamaño de letra.
pub const CHAR_EM: f32 = 0.5;
/// Alto de línea respecto al tamaño de letra.
pub const LINE_EM: f32 = 1.15;
pub const MIN_FONT: f32 = 8.0;
const MAX_FONT: f32 = 28.0;
const MAX_LINES: usize = 3;

/// Escala única y origen para centrar el tablero en un rectángulo.
///
/// `unit` sale del lado que antes se queda corto. Ancho y alto comparten ese
/// valor, así que una tecla de 1u sigue cuadrada.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Place {
    pub unit: f32,
    pub origin_x: f32,
    pub origin_y: f32,
    pub board_w: f32,
    pub board_h: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KeyBox {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Fitted {
    pub text: String,
    pub font: f32,
    pub lines: Vec<String>,
}

impl Fitted {
    pub fn contained(&self, inner_w: f32, inner_h: f32) -> bool {
        lines_fit(&self.lines, self.font, inner_w, inner_h)
    }
}

fn lines_fit(lines: &[String], font: f32, inner_w: f32, inner_h: f32) -> bool {
    !lines.is_empty()
        && lines.len() <= MAX_LINES
        && lines
            .iter()
            .all(|line| !line.is_empty() && line_px(line, font) <= inner_w + 0.05)
        && lines.len() as f32 * font * LINE_EM <= inner_h + 0.05
}

pub fn place(avail_w: f32, avail_h: f32) -> Place {
    let w = if avail_w.is_finite() {
        avail_w.max(1.0)
    } else {
        1.0
    };
    let h = if avail_h.is_finite() {
        avail_h.max(1.0)
    } else {
        1.0
    };
    let unit = (w / COLS).min(h / ROWS);
    let board_w = unit * COLS;
    let board_h = unit * ROWS;
    Place {
        unit,
        origin_x: (w - board_w) * 0.5,
        origin_y: (h - board_h) * 0.5,
        board_w,
        board_h,
    }
}

pub fn key_box(place: Place, key: &KeyGeom) -> KeyBox {
    let gap = place.unit * GAP;
    KeyBox {
        x: place.origin_x + key.x * place.unit + gap * 0.5,
        y: place.origin_y + key.y * place.unit + gap * 0.5,
        w: key.w * place.unit - gap,
        h: place.unit - gap,
    }
}

/// Rectángulo interior de la leyenda, con el mismo margen en los dos ejes.
pub fn legend_inner(key_w: f32, key_h: f32) -> (f32, f32) {
    let pad = key_w.min(key_h) * 0.06;
    ((key_w - pad * 2.0).max(1.0), (key_h - pad * 2.0).max(1.0))
}

/// Área del dibujo en la ventana por defecto: 1180×760, inspector de 380 px,
/// descontando la barra superior y la fila de capas.
pub fn reference_canvas() -> (f32, f32) {
    (1180.0 - 380.0 - 24.0, 760.0 - 96.0)
}

pub fn line_px(line: &str, font: f32) -> f32 {
    line.chars().count() as f32 * font * CHAR_EM
}

/// Parte el texto, baja la letra y, si hace falta, corta con elipsis.
/// El resultado no queda vacío y cabe en el cuadro cuando este admite al menos una línea.
pub fn fit_caption(text: &str, inner_w: f32, inner_h: f32) -> Fitted {
    let source = text.trim();
    if source.is_empty()
        || !inner_w.is_finite()
        || !inner_h.is_finite()
        || inner_w < 1.0
        || inner_h < 1.0
    {
        return finish(vec!["…".to_string()], MIN_FONT);
    }
    let mut font = (inner_h / LINE_EM).clamp(MIN_FONT, MAX_FONT);
    loop {
        if let Some(lines) = wrap_words(source, font, inner_w) {
            let height = lines.len() as f32 * font * LINE_EM;
            if lines.len() <= MAX_LINES && height <= inner_h + 0.05 {
                return finish(lines, font);
            }
        }
        if font <= MIN_FONT + 0.001 {
            let broken = wrap_hard(source, MIN_FONT, inner_w);
            return finish(limit_lines(broken, MIN_FONT, inner_w, inner_h), MIN_FONT);
        }
        font = (font - 0.5).max(MIN_FONT);
    }
}

fn finish(lines: Vec<String>, font: f32) -> Fitted {
    let text = lines.join("\n");
    Fitted { text, font, lines }
}

fn max_chars(font: f32, inner_w: f32) -> usize {
    let width = font * CHAR_EM;
    if width <= 0.0 {
        1
    } else {
        ((inner_w / width).floor() as usize).max(1)
    }
}

/// Parte por palabras. `None` si una palabra no cabe entera en una línea.
fn wrap_words(text: &str, font: f32, inner_w: f32) -> Option<Vec<String>> {
    let limit = max_chars(font, inner_w);
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        if word.chars().count() > limit {
            return None;
        }
        if line.is_empty() {
            line.push_str(word);
            continue;
        }
        if line.chars().count() + 1 + word.chars().count() <= limit {
            line.push(' ');
            line.push_str(word);
        } else {
            lines.push(std::mem::take(&mut line));
            line.push_str(word);
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    if lines.is_empty() {
        return None;
    }
    Some(lines)
}

fn wrap_hard(text: &str, font: f32, inner_w: f32) -> Vec<String> {
    let limit = max_chars(font, inner_w);
    let mut lines = Vec::new();
    let mut line = String::new();
    for ch in text.chars() {
        if ch == ' ' && line.is_empty() {
            continue;
        }
        if line.chars().count() >= limit {
            lines.push(std::mem::take(&mut line));
            if ch == ' ' {
                continue;
            }
        }
        line.push(ch);
    }
    if !line.is_empty() {
        lines.push(line);
    }
    if lines.is_empty() {
        lines.push("…".to_string());
    }
    lines
}

fn limit_lines(mut lines: Vec<String>, font: f32, inner_w: f32, inner_h: f32) -> Vec<String> {
    let max_lines = ((inner_h / (font * LINE_EM)).floor() as usize).clamp(1, MAX_LINES);
    let truncated = lines.len() > max_lines;
    lines.truncate(max_lines);
    if truncated {
        if let Some(last) = lines.last_mut() {
            *last = force_ellipsis(last, font, inner_w);
        }
    }
    for line in &mut lines {
        if line_px(line, font) > inner_w + 0.05 {
            *line = force_ellipsis(line, font, inner_w);
        }
    }
    lines.retain(|line| !line.is_empty());
    if lines.is_empty() {
        lines.push("…".to_string());
    }
    lines
}

fn force_ellipsis(line: &str, font: f32, inner_w: f32) -> String {
    let mut kept = String::new();
    for ch in line.chars().filter(|ch| *ch != '…') {
        let mut next = String::with_capacity(kept.len() + ch.len_utf8() + "…".len());
        next.push_str(&kept);
        next.push(ch);
        next.push('…');
        if line_px(&next, font) > inner_w + 0.05 {
            break;
        }
        kept.push(ch);
    }
    if kept.is_empty() {
        return "…".to_string();
    }
    kept.push('…');
    kept
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sixty_one_keys_and_space_is_index_83() {
        let keys = keys();
        assert_eq!(keys.len(), 61);
        let space = keys.iter().find(|key| key.index == 83).unwrap();
        assert_eq!(space.label, "Espacio");
        assert_eq!(space.hid, 0x2C);
    }
}
