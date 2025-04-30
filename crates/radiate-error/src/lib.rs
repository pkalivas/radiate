use std::{
    borrow::Cow,
    fmt::{self, Display, Formatter},
    ops::Deref,
};
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum RadiateError {
    #[error("Invalid configuration: {0}")]
    InvalidConfig(ErrString),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(ErrString),

    #[error("Invalid data: {0}")]
    EngineError(ErrString),

    #[error("Multiple Errors: {:?}", _0)]
    Multiple(Vec<RadiateError>),
}

impl RadiateError {
    pub fn with_context(self, message: impl Into<String>) -> ErrorContext {
        ErrorContext::new(message).with_source(self)
    }
}

impl From<ErrorContext> for RadiateError {
    fn from(ctx: ErrorContext) -> Self {
        RadiateError::EngineError(ctx.to_string().into())
    }
}

pub type RadiateResult<T> = Result<T, RadiateError>;

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

impl<T> From<T> for ErrString
where
    T: Into<Cow<'static, str>>,
{
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

pub trait IntoRadiateError<T> {
    fn into_radiate_error(self) -> RadiateResult<T>;
}

impl<T, E> IntoRadiateError<T> for Result<T, E>
where
    E: Into<RadiateError>,
{
    fn into_radiate_error(self) -> RadiateResult<T> {
        self.map_err(Into::into)
    }
}

/// Error context for adding additional information to errors
#[derive(Debug)]
pub struct ErrorContext {
    message: String,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl ErrorContext {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            source: None,
        }
    }

    pub fn with_source(mut self, source: impl std::error::Error + Send + Sync + 'static) -> Self {
        self.source = Some(Box::new(source));
        self
    }
}

impl fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(source) = &self.source {
            write!(f, "\nCaused by: {}", source)?;
        }
        Ok(())
    }
}

impl std::error::Error for ErrorContext {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|s| s.as_ref() as _)
    }
}

// ==== radiate-errors macro support ====
#[doc(hidden)]
pub mod __private {
    #[doc(hidden)]
    #[inline]
    #[cold]
    #[must_use]
    pub fn must_use<E>(error: E) -> E {
        error
    }
}

#[macro_export]
macro_rules! radiate_err {
    ($variant:ident: $fmt:literal $(, $arg:expr)* $(,)?) => {
        $crate::__private::must_use(
            $crate::RadiateError::$variant(format!($fmt, $($arg),*).into())
        )
    };
    ($variant:ident: $err:expr $(,)?) => {
        $crate::__private::must_use(
            $crate::RadiateError::$variant($err.into())
        )
    };
}

#[macro_export]
macro_rules! radiate_bail {
    ($($tt:tt)+) => {
        return Err($crate::radiate_err!($($tt)+))
    };
}
