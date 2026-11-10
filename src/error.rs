use std::fmt;

/// Error de configuración o de transporte. El texto está pensado para mostrarse en la ventana.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    ForbiddenCommand(u8),
    OutOfRange {
        field: &'static str,
        detail: String,
    },
    BadLength {
        what: &'static str,
        got: usize,
        expected: usize,
    },
    Protocol(&'static str),
    Json(String),
    Io(String),
    Timeout,
    NoDevice,
    RefusedPid,
    WrongInterface,
    VerifyMismatch {
        cmd: u8,
    },
    NotReady,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ForbiddenCommand(cmd) => write!(f, "comando prohibido 0x{cmd:02X}"),
            Error::OutOfRange { field, detail } => write!(f, "{field} fuera de rango ({detail})"),
            Error::BadLength {
                what,
                got,
                expected,
            } => {
                write!(f, "{what}: largo {got}, se esperaban {expected} bytes")
            }
            Error::Protocol(msg) => write!(f, "{msg}"),
            Error::Json(msg) => write!(f, "JSON: {msg}"),
            Error::Io(msg) => write!(f, "HID: {msg}"),
            Error::Timeout => write!(f, "el teclado no respondió a tiempo"),
            Error::NoDevice => write!(f, "no hay un MINI 60 HE PRO conectado"),
            Error::RefusedPid => {
                write!(
                    f,
                    "el PID 0x80B2 no tiene captura; no se abre ni se escribe"
                )
            }
            Error::WrongInterface => write!(f, "la interfaz HID no es la de configuración"),
            Error::VerifyMismatch { cmd } => {
                write!(
                    f,
                    "la relectura del comando 0x{cmd:02X} no coincide con lo escrito"
                )
            }
            Error::NotReady => {
                write!(f, "lee el teclado o importa un perfil antes de escribir")
            }
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
