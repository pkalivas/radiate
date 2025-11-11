use crate::RadiateError;
use pyo3::PyErr;
use pyo3::exceptions::{PyRuntimeError, PyTypeError, PyValueError};

impl From<RadiateError> for PyErr {
    fn from(e: RadiateError) -> Self {
        match e {
            RadiateError::Builder(message) => PyValueError::new_err(message),
            RadiateError::Codec(message) => PyTypeError::new_err(message),
            RadiateError::Engine(message)
            | RadiateError::Evaluation(message)
            | RadiateError::Genome(message) => PyRuntimeError::new_err(message),
            RadiateError::Fitness(message) => PyRuntimeError::new_err(message),
            RadiateError::Io(source) => PyRuntimeError::new_err(source.to_string()),
            RadiateError::Multiple(m) => PyRuntimeError::new_err(m),
            RadiateError::Context { .. } => PyRuntimeError::new_err(e.to_string()),
            #[cfg(feature = "python")]
            RadiateError::Python(source) => source,
        }
    }
}

#[cfg(feature = "python")]
#[macro_export]
macro_rules! radiate_py_err {
    // Default: ValueError
    ($msg:expr $(,)?) => {{
        pyo3::exceptions::PyValueError::new_err($msg.to_string())
    }};
    // Specify exception class: radiate_py_err!(KeyError, "message {}", x)
    ($exc:ident, $fmt:literal $(, $arg:expr)* $(,)?) => {{
        pyo3::exceptions::$exc::new_err(format!($fmt, $($arg),*))
    }};
    // Specify exception class with raw message expr
    ($exc:ident, $msg:expr $(,)?) => {{
        pyo3::exceptions::$exc::new_err($msg.to_string())
    }};
}

#[cfg(feature = "python")]
#[macro_export]
macro_rules! radiate_py_bail {
    ($($tt:tt)+) => {
        return Err($crate::radiate_py_err!($($tt)+))
    };
}

#[cfg(feature = "python")]
#[macro_export]
macro_rules! radiate_py_ensure {
    ($cond:expr, $($tt:tt)+) => {
        if !$cond {
            $crate::radiate_py_bail!($($tt)+);
        }
    };
}
