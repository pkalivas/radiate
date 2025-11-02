use thiserror::Error;
#[cfg(feature = "python")]
pub mod python;

pub type Result<T> = std::result::Result<T, RadiateError>;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Code {
    InvalidConfig,
    InvalidParameter,
    Engine,
    Codec,
    Evaluation,
    Genome,
    Io,
    Serde,
    ThreadPool,
    Python,
    Multiple,
    Context,
}

#[derive(Error, Debug)]
pub enum RadiateError {
    #[error("Builder error: {message}")]
    Builder { message: String },

    #[error("Engine error: {message}")]
    Engine { message: String },

    #[error("Genome error: {message}")]
    Genome { message: String },

    #[error("Codec error: {message}")]
    Codec { message: String },

    #[error("Evaluation error: {message}")]
    Evaluation { message: String },

    #[error("Invalid parameter: {message}")]
    InvalidParameter { message: String },

    #[error("I/O error: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },

    #[cfg(feature = "python")]
    #[error("Python error: {source}")]
    Python {
        #[from]
        source: pyo3::PyErr,
    },

    #[error("Multiple errors:\n{0}")]
    Multiple(String),

    #[error("{context}\nCaused by: {source}")]
    Context {
        context: String,
        #[source]
        source: Box<RadiateError>,
    },
}

impl RadiateError {
    pub fn code(&self) -> Code {
        match self {
            RadiateError::Builder { .. } => Code::InvalidConfig,
            RadiateError::InvalidParameter { .. } => Code::InvalidParameter,
            RadiateError::Engine { .. } => Code::Engine,
            RadiateError::Genome { .. } => Code::Genome,
            RadiateError::Codec { .. } => Code::Codec,
            RadiateError::Evaluation { .. } => Code::Evaluation,
            RadiateError::Io { .. } => Code::Io,
            #[cfg(feature = "python")]
            RadiateError::Python { .. } => Code::Python,
            RadiateError::Multiple(_) => Code::Multiple,
            RadiateError::Context { .. } => Code::Context,
        }
    }
    pub fn context(self, msg: impl Into<String>) -> Self {
        RadiateError::Context {
            context: msg.into(),
            source: Box::new(self),
        }
    }
}

pub trait ResultExt<T> {
    fn context(self, msg: impl Into<String>) -> Result<T>;

    fn with_context<F: FnOnce() -> String>(self, f: F) -> Result<T>;
}

impl<T, E: Into<RadiateError>> ResultExt<T> for std::result::Result<T, E> {
    fn context(self, msg: impl Into<String>) -> Result<T> {
        self.map_err(|e| e.into().context(msg))
    }

    fn with_context<F: FnOnce() -> String>(self, f: F) -> Result<T> {
        self.map_err(|e| e.into().context(f()))
    }
}

#[doc(hidden)]
pub mod __private {
    #[inline]
    #[cold]
    #[must_use]
    pub fn must_use<E>(e: E) -> E {
        e
    }
}

#[macro_export]
macro_rules! radiate_err {
    // Formatted message
    (Builder: $fmt:literal $(, $arg:expr)* $(,)?) => {
        $crate::__private::must_use($crate::RadiateError::Builder { message: format!($fmt, $($arg),*) })
    };
    (Engine: $fmt:literal $(, $arg:expr)* $(,)?) => {
        $crate::__private::must_use($crate::RadiateError::Engine { message: format!($fmt, $($arg),*) })
    };
    (Genome: $fmt:literal $(, $arg:expr)* $(,)?) => {
        $crate::__private::must_use($crate::RadiateError::Genome { message: format!($fmt, $($arg),*) })
    };
    (Codec: $fmt:literal $(, $arg:expr)* $(,)?) => {
        $crate::__private::must_use($crate::RadiateError::Codec { message: format!($fmt, $($arg),*) })
    };
    (Evaluation: $fmt:literal $(, $arg:expr)* $(,)?) => {
        $crate::__private::must_use($crate::RadiateError::Evaluation { message: format!($fmt, $($arg),*) })
    };
    (InvalidParameter: $fmt:literal $(, $arg:expr)* $(,)?) => {
        $crate::__private::must_use($crate::RadiateError::InvalidParameter { message: format!($fmt, $($arg),*) })
    };
    (Python: $fmt:literal $(, $arg:expr)* $(,)?) => {
        $crate::__private::must_use(pyo3::PyErr::new::<pyo3::exceptions::PyException, _>(format!($fmt, $($arg),*)))
    };

    // Raw string-like message (any expr -> String)
    (Builder: $msg:expr $(,)?) => {
        $crate::__private::must_use($crate::RadiateError::Builder { message: $msg.to_string() })
    };
    (Engine: $msg:expr $(,)?) => {
        $crate::__private::must_use($crate::RadiateError::Engine { message: $msg.to_string() })
    };
    (Genome: $msg:expr $(,)?) => {
        $crate::__private::must_use($crate::RadiateError::Genome { message: $msg.to_string() })
    };
    (Codec: $msg:expr $(,)?) => {
        $crate::__private::must_use($crate::RadiateError::Codec { message: $msg.to_string() })
    };
    (Evaluation: $msg:expr $(,)?) => {
        $crate::__private::must_use($crate::RadiateError::Evaluation { message: $msg.to_string() })
    };
    (InvalidParameter: $msg:expr $(,)?) => {
        $crate::__private::must_use($crate::RadiateError::InvalidParameter { message: $msg.to_string() })
    };

    // Fallback -> Engine
    ($msg:expr $(,)?) => {
        $crate::__private::must_use($crate::RadiateError::Engine { message: $msg.to_string() })
    };
}

#[macro_export]
macro_rules! radiate_bail {
    ($($tt:tt)+) => { return Err($crate::radiate_err!($($tt)+)) };
}

#[macro_export]
macro_rules! ensure {
    ($cond:expr, $($tt:tt)+) => {
        if !$cond { $crate::radiate_bail!($($tt)+); }
    };
}
