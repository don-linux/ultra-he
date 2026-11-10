use crate::error::{Error, Result};

/// Recorrido máximo en centésimas de milímetro (0.01 mm).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Axis {
    pub id: u8,
    pub name: &'static str,
    pub max_cmm: u16,
}

/// Lista viva del driver hec para el MINI 60 HE PRO (`productId` 32930).
pub const AXES: &[Axis] = &[
    Axis {
        id: 1,
        name: "Madera gris",
        max_cmm: 330,
    },
    Axis {
        id: 4,
        name: "Emperador negro",
        max_cmm: 340,
    },
    Axis {
        id: 2,
        name: "Jade Pro",
        max_cmm: 330,
    },
    Axis {
        id: 3,
        name: "Magneto",
        max_cmm: 330,
    },
    Axis {
        id: 5,
        name: "Nube espiritual",
        max_cmm: 340,
    },
    Axis {
        id: 6,
        name: "Nube de humo",
        max_cmm: 340,
    },
    Axis {
        id: 7,
        name: "Nube de viento",
        max_cmm: 340,
    },
    Axis {
        id: 8,
        name: "Nube de hielo",
        max_cmm: 340,
    },
];

pub const JADE_PRO_ID: u8 = 2;
pub const JADE_PRO_MAX: u16 = 330;
pub const JADE_ULTRA_MAX: u16 = 320;
pub const TRIP_MIN: u16 = 10;
pub const RT_MIN: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwitchChoice {
    /// Eje de la lista oficial. El id 2 es Jade Pro.
    Axis(u8),
    /// El firmware no tiene id propio: se escribe 2 y el tope baja a 3.20 mm.
    JadeUltra,
    /// Id leído que el driver actual no nombra (por ejemplo el 0 de un firmware viejo).
    Unlisted(u8),
}

impl SwitchChoice {
    pub fn from_device_byte(id: u8) -> Self {
        if AXES.iter().any(|axis| axis.id == id) {
            SwitchChoice::Axis(id)
        } else {
            SwitchChoice::Unlisted(id)
        }
    }

    pub fn axis_byte(self) -> u8 {
        match self {
            SwitchChoice::Axis(id) | SwitchChoice::Unlisted(id) => id,
            SwitchChoice::JadeUltra => JADE_PRO_ID,
        }
    }

    pub fn max_cmm(self) -> Result<u16> {
        match self {
            SwitchChoice::JadeUltra => Ok(JADE_ULTRA_MAX),
            SwitchChoice::Axis(id) => AXES
                .iter()
                .find(|axis| axis.id == id)
                .map(|axis| axis.max_cmm)
                .ok_or_else(|| Error::OutOfRange {
                    field: "eje",
                    detail: format!("id {id} no está en la lista del driver"),
                }),
            SwitchChoice::Unlisted(0) => Ok(340),
            SwitchChoice::Unlisted(id) => Err(Error::OutOfRange {
                field: "eje",
                detail: format!("id {id} no está en la lista del driver"),
            }),
        }
    }

    pub fn label(self) -> String {
        match self {
            SwitchChoice::JadeUltra => "Jade Ultra".to_string(),
            SwitchChoice::Axis(id) => AXES
                .iter()
                .find(|axis| axis.id == id)
                .map(|axis| axis.name.to_string())
                .unwrap_or_else(|| format!("eje {id}")),
            SwitchChoice::Unlisted(id) => format!("id {id}, sin nombre en el driver actual"),
        }
    }

    pub fn profile_key(self) -> String {
        match self {
            SwitchChoice::JadeUltra => "jade-ultra".to_string(),
            SwitchChoice::Axis(id) => format!("axis:{id}"),
            SwitchChoice::Unlisted(id) => format!("unlisted:{id}"),
        }
    }

    pub fn parse_profile(text: &str) -> Result<Self> {
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
