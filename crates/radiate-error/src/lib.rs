use std::borrow::Cow;
use std::fmt::{self, Display, Formatter};
use std::ops::Deref;

#[cfg(feature = "backtrace")]
use std::backtrace::Backtrace;

pub type RadiateResult<T> = Result<T, RadiateError>;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ErrorCode {
    InvalidConfig,
    InvalidParameter,
    Engine,
    Codec,
    Evaluation,
    Io,
    Serde,
    ThreadPool,
    Python,
    Multiple,
    Context,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrString(Cow<'static, str>);

impl AsRef<str> for ErrString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for ErrString {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for ErrString {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T: Into<Cow<'static, str>>> From<T> for ErrString {
    fn from(v: T) -> Self {
        Self(v.into())
    }
}

#[derive(Debug)]
pub enum RadiateError {
    InvalidConfig {
        message: ErrString,
    },
    InvalidParameter {
        message: ErrString,
    },
    Engine {
        message: ErrString,
    },
    Codec {
        message: ErrString,
    },
    Evaluation {
        message: ErrString,
    },

    #[cfg(feature = "python")]
    Python {
        source: pyo3::PyErr,
        #[cfg(feature = "backtrace")]
        backtrace: Backtrace,
    },

    Multiple(MultiDisplay),

    Context {
        context: ErrorContext,
        source: Box<RadiateError>,
        #[cfg(feature = "backtrace")]
        backtrace: Backtrace,
    },
}

impl RadiateError {
    pub fn code(&self) -> ErrorCode {
        match self {
            Self::InvalidConfig { .. } => ErrorCode::InvalidConfig,
            Self::InvalidParameter { .. } => ErrorCode::InvalidParameter,
            Self::Engine { .. } => ErrorCode::Engine,
            Self::Codec { .. } => ErrorCode::Codec,
            Self::Evaluation { .. } => ErrorCode::Evaluation,
            #[cfg(feature = "python")]
            Self::Python { .. } => ErrorCode::Python,
            Self::Multiple(_) => ErrorCode::Multiple,
            Self::Context { .. } => ErrorCode::Context,
        }
    }

    pub fn with_context(self, msg: impl Into<String>) -> Self {
        RadiateError::Context {
            context: ErrorContext::new(msg),
            source: Box::new(self),
            #[cfg(feature = "backtrace")]
            backtrace: Backtrace::capture(),
        }
    }
}

impl Display for RadiateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfig { message } => write!(f, "Invalid configuration: {}", message),
            Self::InvalidParameter { message } => write!(f, "Invalid parameter: {}", message),
            Self::Engine { message } => write!(f, "Engine error: {}", message),
            Self::Codec { message } => write!(f, "Codec error: {}", message),
            Self::Evaluation { message } => write!(f, "Evaluation error: {}", message),
            #[cfg(feature = "python")]
            Self::Python { source, .. } => write!(f, "Python error: {}", source),
            Self::Multiple(m) => write!(f, "Multiple errors:\n{}", m),
            Self::Context {
                context, source, ..
            } => write!(f, "{}\nCaused by: {}", context, source),
        }
    }
}

impl std::error::Error for RadiateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            #[cfg(feature = "python")]
            Self::Python { source, .. } => Some(source),
            Self::Context { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct ErrorContext {
    message: String,
}
impl ErrorContext {
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
        }
    }
}
impl Display for ErrorContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl std::error::Error for ErrorContext {}

#[derive(Debug)]
pub struct MultiDisplay(Vec<RadiateError>);
impl Display for MultiDisplay {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for (i, e) in self.0.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "[{}] {} (code: {:?})", i, e, e.code())?;
        }
        Ok(())
    }
}
impl From<Vec<RadiateError>> for MultiDisplay {
    fn from(v: Vec<RadiateError>) -> Self {
        Self(v)
    }
}

#[cfg(feature = "python")]
impl From<pyo3::PyErr> for RadiateError {
    fn from(source: pyo3::PyErr) -> Self {
        RadiateError::Python {
            source,
            #[cfg(feature = "backtrace")]
            backtrace: Backtrace::capture(),
        }
    }
}

// Ergonomic Result extensions (context)
pub trait ResultExt<T> {
    fn context(self, msg: impl Into<String>) -> RadiateResult<T>;
    fn with_context<F: FnOnce() -> String>(self, f: F) -> RadiateResult<T>;
}

impl<T, E: Into<RadiateError>> ResultExt<T> for Result<T, E> {
    fn context(self, msg: impl Into<String>) -> RadiateResult<T> {
        self.map_err(|e| e.into().with_context(msg))
    }

    fn with_context<F: FnOnce() -> String>(self, f: F) -> RadiateResult<T> {
        self.map_err(|e| e.into().with_context(f()))
    }
}

// Bridge for existing code
pub trait IntoRadiateError<T> {
    fn into_radiate_error(self) -> RadiateResult<T>;
}
impl<T, E: Into<RadiateError>> IntoRadiateError<T> for Result<T, E> {
    fn into_radiate_error(self) -> RadiateResult<T> {
        self.map_err(Into::into)
    }
}

// Macros: err, bail, ensure
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
    (InvalidConfig: $fmt:literal $(, $arg:expr)* $(,)?) => {
        $crate::__private::must_use($crate::RadiateError::InvalidConfig { message: format!($fmt, $($arg),*).into() })
    };
    (InvalidParameter: $fmt:literal $(, $arg:expr)* $(,)?) => {
        $crate::__private::must_use($crate::RadiateError::InvalidParameter { message: format!($fmt, $($arg),*).into() })
    };
    (Engine: $fmt:literal $(, $arg:expr)* $(,)?) => {
        $crate::__private::must_use($crate::RadiateError::Engine { message: format!($fmt, $($arg),*).into() })
    };
    (Codec: $fmt:literal $(, $arg:expr)* $(,)?) => {
        $crate::__private::must_use($crate::RadiateError::Codec { message: format!($fmt, $($arg),*).into() })
    };
    (Evaluation: $fmt:literal $(, $arg:expr)* $(,)?) => {
        $crate::__private::must_use($crate::RadiateError::Evaluation { message: format!($fmt, $($arg),*).into() })
    };
    ($variant:ident: $msg:expr $(,)?) => {{
        // Fallback to Engine with custom message if variant omitted
        $crate::__private::must_use($crate::RadiateError::Engine { message: $msg.into() })
    }};
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

// use std::{
//     borrow::Cow,
//     fmt::{self, Display, Formatter},
//     ops::Deref,
// };

// #[derive(Debug, Clone, PartialEq, Eq)]
// pub enum RadiateError {
//     InvalidConfig(ErrString),
//     InvalidParameter(ErrString),

//     EngineError(ErrString),

//     Multiple(Vec<RadiateError>),
// }

// impl RadiateError {
//     pub fn with_context(self, message: impl Into<String>) -> ErrorContext {
//         ErrorContext::new(message).with_source(self)
//     }
// }

// impl From<ErrorContext> for RadiateError {
//     fn from(ctx: ErrorContext) -> Self {
//         RadiateError::EngineError(ctx.to_string().into())
//     }
// }

// pub type RadiateResult<T> = Result<T, RadiateError>;

// #[derive(Debug, Clone, PartialEq, Eq)]
// pub struct ErrString(Cow<'static, str>);

// impl AsRef<str> for ErrString {
//     fn as_ref(&self) -> &str {
//         &self.0
//     }
// }

// impl Deref for ErrString {
//     type Target = str;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl Display for ErrString {
//     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
//         write!(f, "{}", self.0)
//     }
// }

// impl<T> From<T> for ErrString
// where
//     T: Into<Cow<'static, str>>,
// {
//     fn from(value: T) -> Self {
//         Self(value.into())
//     }
// }

// pub trait IntoRadiateError<T> {
//     fn into_radiate_error(self) -> RadiateResult<T>;
// }

// impl<T, E> IntoRadiateError<T> for Result<T, E>
// where
//     E: Into<RadiateError>,
// {
//     fn into_radiate_error(self) -> RadiateResult<T> {
//         self.map_err(Into::into)
//     }
// }

// /// Error context for adding additional information to errors
// #[derive(Debug)]
// pub struct ErrorContext {
//     message: String,
//     source: Option<Box<dyn std::error::Error + Send + Sync>>,
// }

// impl ErrorContext {
//     pub fn new(message: impl Into<String>) -> Self {
//         Self {
//             message: message.into(),
//             source: None,
//         }
//     }

//     pub fn with_source(mut self, source: impl std::error::Error + Send + Sync + 'static) -> Self {
//         self.source = Some(Box::new(source));
//         self
//     }
// }

// impl fmt::Display for ErrorContext {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "{}", self.message)?;
//         if let Some(source) = &self.source {
//             write!(f, "\nCaused by: {}", source)?;
//         }
//         Ok(())
//     }
// }

// impl std::error::Error for ErrorContext {
//     fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
//         self.source.as_ref().map(|s| s.as_ref() as _)
//     }
// }

// // ==== radiate-errors macro support ====
// #[doc(hidden)]
// pub mod __private {
//     #[doc(hidden)]
//     #[inline]
//     #[cold]
//     #[must_use]
//     pub fn must_use<E>(error: E) -> E {
//         error
//     }
// }

// #[macro_export]
// macro_rules! radiate_err {
//     ($variant:ident: $fmt:literal $(, $arg:expr)* $(,)?) => {
//         $crate::__private::must_use(
//             $crate::RadiateError::$variant(format!($fmt, $($arg),*).into())
//         )
//     };
//     ($variant:ident: $err:expr $(,)?) => {
//         $crate::__private::must_use(
//             $crate::RadiateError::$variant($err.into())
//         )
//     };
// }

// #[macro_export]
// macro_rules! radiate_bail {
//     ($($tt:tt)+) => {
//         return Err($crate::radiate_err!($($tt)+))
//     };
// }
