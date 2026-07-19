use crate::{Code, RadiateError};
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
            RadiateError::Metric(message) => PyRuntimeError::new_err(message),
            RadiateError::Expr(message) => PyRuntimeError::new_err(message),
            RadiateError::Other(message) => PyRuntimeError::new_err(message),
            RadiateError::AnyValue(message) => PyRuntimeError::new_err(message),
            RadiateError::Multiple(m) => PyRuntimeError::new_err(m),
            // Context wraps another error; classify by the root cause so the
            // exception type reflects what went wrong, while the message keeps
            // the full "context\nCaused by: ..." chain.
            RadiateError::Context { .. } => {
                let message = e.to_string();
                match e.leaf_code() {
                    Code::InvalidConfig => PyValueError::new_err(message),
                    Code::Codec => PyTypeError::new_err(message),
                    _ => PyRuntimeError::new_err(message),
                }
            }
            #[cfg(feature = "python")]
            RadiateError::Python(source) => source,
            RadiateError::IO(source) => PyRuntimeError::new_err(format!("I/O error: {}", source)),
            RadiateError::Fmt(source) => {
                PyRuntimeError::new_err(format!("Formatting error: {}", source))
            }
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
