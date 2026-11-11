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
