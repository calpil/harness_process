use std::fmt;

/// Equivalente de `SystemExit` de Python: mensaje opcional (a stderr, sin
/// prefijo) + exit code. `SystemExit("texto")` -> code 1 con mensaje;
/// `SystemExit(2)` -> code 2 silencioso.
#[derive(Debug)]
pub struct Exit {
    pub code: i32,
    pub message: Option<String>,
}

impl Exit {
    pub fn msg(message: impl Into<String>) -> Self {
        Exit {
            code: 1,
            message: Some(message.into()),
        }
    }

    pub fn code(code: i32) -> Self {
        Exit {
            code,
            message: None,
        }
    }
}

impl fmt::Display for Exit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.message {
            Some(m) => write!(f, "{m}"),
            None => write!(f, "exit {}", self.code),
        }
    }
}

impl std::error::Error for Exit {}
