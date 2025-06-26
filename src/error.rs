use std::error::Error;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum BseError {
    DataNotFound(String),
    DataError(String),
    KeyError(String),
    ValueError(String),
    IOError(String),
    SerdeJsonError(String),
    Miscellaneous(String),
    UninitializedFieldError(derive_builder::UninitializedFieldError),
}

impl Display for BseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Error for BseError {}

impl From<std::io::Error> for BseError {
    fn from(err: std::io::Error) -> Self {
        BseError::IOError(err.to_string())
    }
}

impl From<serde_json::Error> for BseError {
    fn from(err: serde_json::Error) -> Self {
        BseError::SerdeJsonError(err.to_string())
    }
}

impl From<derive_builder::UninitializedFieldError> for BseError {
    fn from(err: derive_builder::UninitializedFieldError) -> Self {
        BseError::UninitializedFieldError(err)
    }
}

#[macro_export]
macro_rules! bse_trace {
    () => {
        concat!(file!(), ":", line!(), ":", column!(), ": ")
    };
}

#[macro_export]
macro_rules! bse_raise {
    ($errtype: ident, $($arg:tt)*) => {{
        use $crate::prelude::*;
        use std::fmt::Write;
        let mut s = String::new();
        write!(s, bse_trace!()).unwrap();
        write!(s, concat!("BseError::", stringify!($errtype), ": ")).unwrap();
        write!(s, $($arg)*).unwrap();
        Err(BseError::$errtype(s))
    }};
}
