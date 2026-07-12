use thiserror::Error;
#[cfg(feature = "python")]
pub mod python;

pub type Result<T> = std::result::Result<T, RadiateError>;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Code {
    InvalidConfig,
    Engine,
    Codec,
    Evaluation,
    Genome,
    Fitness,
    Metric,
    Expr,
    AnyValue,
    Other,
    Io,
    Python,
    Multiple,
    Context,
    IO,
}

#[derive(Error, Debug)]
pub enum RadiateError {
    #[error("Builder error: {0}")]
    Builder(String),

    #[error("Engine error: {0}")]
    Engine(String),

    #[error("Genome error: {0}")]
    Genome(String),

    #[error("Codec error: {0}")]
    Codec(String),

    #[error("Evaluation error: {0}")]
    Evaluation(String),

    #[error("Invalid fitness: {0}")]
    Fitness(String),

    #[error("Metric error: {0}")]
    Metric(String),

    #[error("Expression error: {0}")]
    Expr(String),

    #[cfg(feature = "python")]
    #[error("Python error: {0}")]
    Python(#[from] pyo3::PyErr),

    #[error("Multiple errors:\n{0}")]
    Multiple(String),

    #[error("AnyValue error: {0}")]
    AnyValue(String),

    #[error("Other error: {0}")]
    Other(String),

    #[error("{context}\nCaused by: {source}")]
    Context {
        context: String,
        #[source]
        source: Box<RadiateError>,
    },

    #[error("I/O error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Formatting error: {0}")]
    Fmt(#[from] std::fmt::Error),
}

impl RadiateError {
    pub fn code(&self) -> Code {
        match self {
            RadiateError::Builder { .. } => Code::InvalidConfig,
            RadiateError::Engine { .. } => Code::Engine,
            RadiateError::Genome { .. } => Code::Genome,
            RadiateError::Codec { .. } => Code::Codec,
            RadiateError::Fitness { .. } => Code::Fitness,
            RadiateError::Metric { .. } => Code::Metric,
            RadiateError::Expr { .. } => Code::Expr,
            RadiateError::Evaluation { .. } => Code::Evaluation,
            RadiateError::AnyValue { .. } => Code::AnyValue,
            RadiateError::Other(_) => Code::Other,
            #[cfg(feature = "python")]
            RadiateError::Python { .. } => Code::Python,
            RadiateError::Multiple(_) => Code::Multiple,
            RadiateError::Context { .. } => Code::Context,
            RadiateError::IO(_) => Code::IO,
            RadiateError::Fmt(_) => Code::Other,
        }
    }
    pub fn context(self, msg: impl Into<String>) -> Self {
        RadiateError::Context {
            context: msg.into(),
            source: Box::new(self),
        }
    }

    /// The `Code` of the innermost cause, seeing through `Context` wrappers.
    ///
    /// `code()` reports `Context` for any error given context, which loses the
    /// original classification. This walks to the root cause so callers (e.g.
    /// the `PyErr` conversion) can decide how to surface the error based on what
    /// actually went wrong, not on whether context happened to be attached.
    pub fn leaf_code(&self) -> Code {
        match self {
            RadiateError::Context { source, .. } => source.leaf_code(),
            other => other.code(),
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
        $crate::__private::must_use($crate::RadiateError::Builder(format!($fmt, $($arg),*)))
    };
    (Engine: $fmt:literal $(, $arg:expr)* $(,)?) => {
        $crate::__private::must_use($crate::RadiateError::Engine(format!($fmt, $($arg),*)))
    };
    (Genome: $fmt:literal $(, $arg:expr)* $(,)?) => {
        $crate::__private::must_use($crate::RadiateError::Genome(format!($fmt, $($arg),*)))
    };
    (Codec: $fmt:literal $(, $arg:expr)* $(,)?) => {
        $crate::__private::must_use($crate::RadiateError::Codec(format!($fmt, $($arg),*)))
    };
    (Evaluation: $fmt:literal $(, $arg:expr)* $(,)?) => {
        $crate::__private::must_use($crate::RadiateError::Evaluation(format!($fmt, $($arg),*)))
    };
    (Python: $fmt:literal $(, $arg:expr)* $(,)?) => {
        $crate::__private::must_use(pyo3::PyErr::new::<pyo3::exceptions::PyException, _>(format!($fmt, $($arg),*)))
    };
    (Metric: $fmt:literal $(, $arg:expr)* $(,)?) => {
        $crate::__private::must_use($crate::RadiateError::Metric(format!($fmt, $($arg),*)))
    };
    (Expr: $fmt:literal $(, $arg:expr)* $(,)?) => {
        $crate::__private::must_use($crate::RadiateError::Expr(format!($fmt, $($arg),*)))
    };
    (AnyValue: $fmt:literal $(, $arg:expr)* $(,)?) => {
        $crate::__private::must_use($crate::RadiateError::AnyValue(format!($fmt, $($arg),*)))
    };

    // Contextual message
    (Context: $msg:expr, $source:expr $(,)?) => {
        $crate::__private::must_use($source.into().context($msg))
    };

    (IO: $fmt:literal $(, $arg:expr)* $(,)?) => {
        $crate::__private::must_use($crate::RadiateError::IO(format!($fmt, $($arg),*)))
    };
    (Fmt: $fmt:literal $(, $arg:expr)* $(,)?) => {
        $crate::__private::must_use($crate::RadiateError::Fmt(format!($fmt, $($arg),*)))
    };

    // Raw string-like message (any expr -> String)
    (Builder: $msg:expr $(,)?) => {
        $crate::__private::must_use($crate::RadiateError::Builder($msg.to_string()))
    };
    (Engine: $msg:expr $(,)?) => {
        $crate::__private::must_use($crate::RadiateError::Engine($msg.to_string()))
    };
    (Genome: $msg:expr $(,)?) => {
        $crate::__private::must_use($crate::RadiateError::Genome($msg.to_string()))
    };
    (Codec: $msg:expr $(,)?) => {
        $crate::__private::must_use($crate::RadiateError::Codec($msg.to_string()))
    };
    (Evaluation: $msg:expr $(,)?) => {
        $crate::__private::must_use($crate::RadiateError::Evaluation($msg.to_string()))
    };
    (Python: $msg:expr $(,)?) => {
        $crate::__private::must_use(pyo3::PyErr::new::<pyo3::exceptions::PyException, _>($msg.to_string()))
    };
    (Metric: $msg:expr $(,)?) => {
        $crate::__private::must_use($crate::RadiateError::Metric($msg.to_string()))
    };
    (Expr: $msg:expr $(,)?) => {
        $crate::__private::must_use($crate::RadiateError::Expr($msg.to_string()))
    };
    (IO: $msg:expr $(,)?) => {
        $crate::__private::must_use($crate::RadiateError::IO($msg.to_string()))
    };
    (Fmt: $msg:expr $(,)?) => {
        $crate::__private::must_use($crate::RadiateError::Fmt($msg.to_string()))
    };

    // Fallback -> Engine (for now, could be Metric or other)
    ($msg:expr $(,)?) => {
        $crate::__private::must_use($crate::RadiateError::Engine($msg.to_string()))
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
