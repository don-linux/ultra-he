use crate::error::{Error, Result};

/// Recorrido máximo en centésimas de milímetro (0.01 mm).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Axis {
    pub id: u8,
    /// Clave de hec (`axis_11`, `jade`, …).
    pub text: &'static str,
    pub name: &'static str,
    pub wired_max: u16,
    pub dongle_max: u16,
}

/// Lista viva del driver hec para el MINI 60 HE PRO (`productId` 32930 y su receptor).
/// El tope del id 1 sube a 3.40 mm por el receptor; el resto coincide en los dos enlaces.
pub const AXES: &[Axis] = &[
    Axis {
        id: 1,
        text: "axis_6",
        name: "Madera gris",
        wired_max: 330,
        dongle_max: 340,
    },
    Axis {
        id: 4,
        text: "axis_7",
        name: "Emperador negro",
        wired_max: 340,
        dongle_max: 340,
    },
    Axis {
        id: 2,
        text: "jade",
        name: "Jade Pro",
        wired_max: 330,
        dongle_max: 330,
    },
    Axis {
        id: 3,
        text: "king",
        name: "Magneto",
        wired_max: 330,
        dongle_max: 330,
    },
    Axis {
        id: 5,
        text: "axis_10",
        name: "Nube espiritual",
        wired_max: 340,
        dongle_max: 340,
    },
    Axis {
        id: 6,
        text: "axis_11",
        name: "Nube de humo",
        wired_max: 340,
        dongle_max: 340,
    },
    Axis {
        id: 7,
        text: "axis_12",
        name: "Nube de viento",
        wired_max: 340,
        dongle_max: 340,
    },
    Axis {
        id: 8,
        text: "axis_35",
        name: "Nube de hielo",
        wired_max: 340,
        dongle_max: 340,
    },
];

pub const JADE_PRO_ID: u8 = 2;
pub const JADE_PRO_MAX: u16 = 330;
pub const JADE_ULTRA_MAX: u16 = 320;
pub const TRIP_MIN: u16 = 10;
pub const RT_MIN: u16 = 1;
/// `defaultValue` del MINI 60 HE PRO cuando `AxialModel` no coincide con la lista.
pub const FALLBACK_AXIS: u8 = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwitchSpec {
    pub name: &'static str,
    pub travel_mm: &'static str,
    pub start_gf: u16,
    pub bottom_gf: u16,
    pub flux_start: u16,
    pub flux_bottom: u16,
    pub firmware_cap: u16,
}

pub const JADE_PRO: SwitchSpec = SwitchSpec {
    name: "Gateron Magnetic Jade Pro",
    travel_mm: "3.50",
    start_gf: 36,
    bottom_gf: 50,
    flux_start: 120,
    flux_bottom: 700,
    firmware_cap: JADE_PRO_MAX,
};

pub const JADE_ULTRA: SwitchSpec = SwitchSpec {
    name: "Gateron Magnetic Jade Ultra",
    travel_mm: "3.20",
    start_gf: 30,
    bottom_gf: 40,
    flux_start: 122,
    flux_bottom: 550,
    firmware_cap: JADE_ULTRA_MAX,
};

/// Fila de `config.driveall.cn/axial.json`. El `text` se cruza con [`AXES`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AxialModel {
    pub model: u16,
    pub text: &'static str,
    pub name_es: &'static str,
}

pub const AXIAL_MODELS: &[AxialModel] = &[
    AxialModel {
        model: 3,
        text: "axis_16",
        name_es: "Eje del Rey Celestial",
    },
    AxialModel {
        model: 4,
        text: "axis_13",
        name_es: "Eje del Rey Guerrero",
    },
    AxialModel {
        model: 5,
        text: "axis_9",
        name_es: "Eje Magnético de Arroz",
    },
    AxialModel {
        model: 6,
        text: "axis_23",
        name_es: "Eje Magnético Silencioso",
    },
    AxialModel {
        model: 7,
        text: "green",
        name_es: "Eje Magnético Verde",
    },
    AxialModel {
        model: 8,
        text: "axis_24",
        name_es: "Eje Silencioso",
    },
    AxialModel {
        model: 9,
        text: "panda",
        name_es: "Eje Panda",
    },
    AxialModel {
        model: 10,
        text: "axis_4",
        name_es: "Eje de Cristal de Melocotón",
    },
    AxialModel {
        model: 11,
        text: "key_v13",
        name_es: "Eje del Polo Norte de Haimu",
    },
    AxialModel {
        model: 12,
        text: "key_v14",
        name_es: "Eje del Polo Norte de Hejin",
    },
    AxialModel {
        model: 13,
        text: "axis_25",
        name_es: "Eje Amarillo-Azul de Hejin",
    },
    AxialModel {
        model: 14,
        text: "axis_26",
        name_es: "Interruptor Magnético de Escarcha",
    },
    AxialModel {
        model: 15,
        text: "axis_11",
        name_es: "Eje de Nube de Humo",
    },
    AxialModel {
        model: 16,
        text: "axis_12",
        name_es: "Eje de Nube Tormentosa",
    },
    AxialModel {
        model: 17,
        text: "axis_1",
        name_es: "Eje Magnético del Emperador",
    },
    AxialModel {
        model: 18,
        text: "key_v10",
        name_es: "Eje de Madera Gris",
    },
    AxialModel {
        model: 19,
        text: "key_v8",
        name_es: "Eje de Jade Blanco",
    },
    AxialModel {
        model: 20,
        text: "key_v9",
        name_es: "Eje Azul Soñador",
    },
    AxialModel {
        model: 21,
        text: "key_v7",
        name_es: "Eje Xingyi",
    },
    AxialModel {
        model: 22,
        text: "axis_27",
        name_es: "Eje Cielo Azul",
    },
    AxialModel {
        model: 23,
        text: "axis_10",
        name_es: "Eje Nube Espiritual",
    },
    AxialModel {
        model: 24,
        text: "axis_20",
        name_es: "Eje del Emperador del Mar",
    },
    AxialModel {
        model: 25,
        text: "axis_21",
        name_es: "Eje Alma de Hielo",
    },
    AxialModel {
        model: 26,
        text: "axis_29",
        name_es: "Eje de Dominio Cristalino",
    },
    AxialModel {
        model: 27,
        text: "axis_30",
        name_es: "Eje del Emperador de Fuego",
    },
    AxialModel {
        model: 28,
        text: "axis_31",
        name_es: "Eje de Polvo Magnético",
    },
    AxialModel {
        model: 29,
        text: "axis_7",
        name_es: "Eje de Polvo Magnético",
    },
];

pub fn axial_model(model: u16) -> Option<&'static AxialModel> {
    AXIAL_MODELS.iter().find(|entry| entry.model == model)
}

/// Id de eje del MINI 60 que corresponde a la clave de `axial.json`.
pub fn axis_id_for_text(text: &str) -> Option<u8> {
    match text {
        "axis_6" | "key_v10" => Some(1),
        "jade" => Some(2),
        "king" => Some(3),
        "axis_7" => Some(4),
        "axis_10" => Some(5),
        "axis_11" => Some(6),
        "axis_12" => Some(7),
        "axis_35" => Some(8),
        _ => None,
    }
}

/// Eje de fábrica. Si el modelo no está en la lista del teclado, hec usa 6.
pub fn default_axis_for_model(model: u16) -> u8 {
    axial_model(model)
        .and_then(|entry| axis_id_for_text(entry.text))
        .unwrap_or(FALLBACK_AXIS)
}

pub fn axis_by_id(id: u8) -> Option<&'static Axis> {
    AXES.iter().find(|axis| axis.id == id)
}

pub fn axis_name(id: u8) -> &'static str {
    axis_by_id(id).map(|axis| axis.name).unwrap_or("eje")
}

pub fn axis_cap(id: u8, dongle: bool) -> Result<u16> {
    let axis = axis_by_id(id).ok_or_else(|| Error::OutOfRange {
        field: "eje",
        detail: format!("id {id} no está en la lista del driver"),
    })?;
    Ok(if dongle {
        axis.dongle_max
    } else {
        axis.wired_max
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwitchChoice {
    /// Byte 0: el firmware no nombra el eje y hec usa el de fábrica.
    Default,
    /// Eje de la lista oficial. El id 2 es Jade Pro.
    Axis(u8),
    /// El firmware no tiene id propio: se escribe 2 y el tope baja a 3.20 mm.
    JadeUltra,
    /// Id leído que el driver actual no nombra.
    Unlisted(u8),
}

impl SwitchChoice {
    pub fn from_device_byte(id: u8) -> Self {
        if id == 0 {
            SwitchChoice::Default
        } else if AXES.iter().any(|axis| axis.id == id) {
            SwitchChoice::Axis(id)
        } else {
            SwitchChoice::Unlisted(id)
        }
    }

    pub fn axis_byte(self) -> u8 {
        match self {
            SwitchChoice::Default => 0,
            SwitchChoice::Axis(id) | SwitchChoice::Unlisted(id) => id,
            SwitchChoice::JadeUltra => JADE_PRO_ID,
        }
    }

    /// Tope con el enlace por cable y el eje de fábrica 6. La ventana usa [`Self::cap`].
    pub fn max_cmm(self) -> Result<u16> {
        self.cap(false, FALLBACK_AXIS)
    }

    pub fn cap(self, dongle: bool, default_axis: u8) -> Result<u16> {
        match self {
            SwitchChoice::JadeUltra => Ok(JADE_ULTRA_MAX),
            SwitchChoice::Default => axis_cap(default_axis, dongle),
            SwitchChoice::Axis(id) => axis_cap(id, dongle),
            SwitchChoice::Unlisted(_) => Ok(340),
        }
    }

    /// El mayor tope entre cable y receptor. Un archivo no sabe por dónde se leyó.
    pub fn widest_cap(self, default_axis: u8) -> Result<u16> {
        Ok(self
            .cap(false, default_axis)?
            .max(self.cap(true, default_axis)?))
    }

    pub fn label(self) -> String {
        self.label_with(FALLBACK_AXIS)
    }

    pub fn label_with(self, default_axis: u8) -> String {
        match self {
            SwitchChoice::Default => format!("De fábrica ({})", axis_name(default_axis)),
            SwitchChoice::JadeUltra => "Jade Ultra".to_string(),
            SwitchChoice::Axis(id) => axis_name(id).to_string(),
            SwitchChoice::Unlisted(id) => format!("id {id}, sin nombre en el driver actual"),
        }
    }

    pub fn tint(self) -> [u8; 3] {
        match self {
            SwitchChoice::Default => [196, 168, 96],
            SwitchChoice::JadeUltra => [72, 214, 146],
            SwitchChoice::Axis(1) => [148, 132, 104],
            SwitchChoice::Axis(2) => [32, 150, 96],
            SwitchChoice::Axis(3) => [176, 64, 72],
            SwitchChoice::Axis(4) => [48, 48, 52],
            SwitchChoice::Axis(5) => [168, 140, 214],
            SwitchChoice::Axis(6) => [120, 122, 128],
            SwitchChoice::Axis(7) => [86, 138, 186],
            SwitchChoice::Axis(8) => [176, 214, 232],
            SwitchChoice::Axis(_) | SwitchChoice::Unlisted(_) => [150, 150, 150],
        }
    }

    pub fn profile_key(self) -> String {
        match self {
            SwitchChoice::Default => "default".to_string(),
            SwitchChoice::JadeUltra => "jade-ultra".to_string(),
            SwitchChoice::Axis(id) => format!("axis:{id}"),
            SwitchChoice::Unlisted(id) => format!("unlisted:{id}"),
        }
    }

    pub fn parse_profile(text: &str) -> Result<Self> {
        if text == "default" {
            return Ok(SwitchChoice::Default);
        }
        if text == "jade-ultra" {
            return Ok(SwitchChoice::JadeUltra);
        }
        if text == "jade-pro" {
            return Ok(SwitchChoice::Axis(JADE_PRO_ID));
        }
        if let Some(rest) = text.strip_prefix("axis:") {
            let id: u8 = rest
                .parse()
                .map_err(|_| Error::Json(format!("eje {text}")))?;
            if !AXES.iter().any(|axis| axis.id == id) {
                return Err(Error::OutOfRange {
                    field: "eje",
                    detail: format!("id {id} no está en la lista del driver"),
                });
            }
            return Ok(SwitchChoice::Axis(id));
        }
        if let Some(rest) = text.strip_prefix("unlisted:") {
            let id: u8 = rest
                .parse()
                .map_err(|_| Error::Json(format!("eje {text}")))?;
            return Ok(SwitchChoice::Unlisted(id));
        }
        Err(Error::Json(format!("perfil de eje desconocido: {text}")))
    }
}

pub fn check_trip(choice: SwitchChoice, cmm: u16) -> Result<()> {
    check_span("actuación", choice, cmm, TRIP_MIN)
}

pub fn check_rt(choice: SwitchChoice, cmm: u16) -> Result<()> {
    check_span("rapid trigger", choice, cmm, RT_MIN)
}

fn check_span(field: &'static str, choice: SwitchChoice, cmm: u16, min: u16) -> Result<()> {
    let max = choice.max_cmm()?;
    if cmm < min || cmm > max {
        return Err(Error::OutOfRange {
            field,
            detail: format!("{cmm} centésimas, permitido {min}..={max}"),
        });
    }
    Ok(())
}
