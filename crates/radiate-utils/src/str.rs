use crate::ToSnakeCase;
use std::borrow::Cow;

type Inner = compact_str::CompactString;

#[derive(Clone, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
pub struct SmallStr(Inner);

impl SmallStr {
    pub const EMPTY: Self = Self::from_static("");
    pub const EMPTY_REF: &'static Self = &Self::from_static("");

    #[inline(always)]
    pub const fn from_static(s: &'static str) -> Self {
        Self(Inner::const_new(s))
    }

    #[inline(always)]
    pub fn from_string(s: String) -> Self {
        Self(Inner::from(s))
    }

    #[inline(always)]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    #[inline(always)]
    pub fn as_mut_str(&mut self) -> &mut str {
        self.0.as_mut_str()
    }

    #[inline(always)]
    #[allow(clippy::inherent_to_string_shadow_display)] // This is faster.
    pub fn to_string(&self) -> String {
        self.0.as_str().to_owned()
    }

    #[inline(always)]
    pub fn into_string(self) -> String {
        self.0.into_string()
    }
}

impl Default for SmallStr {
    #[inline(always)]
    fn default() -> Self {
        Self::EMPTY
    }
}

impl ToSnakeCase<SmallStr> for SmallStr {
    fn to_snake_case(&self) -> SmallStr {
        self.as_str().to_snake_case().into()
    }
}

// AsRef, Borrow, Deref impls

impl AsRef<str> for SmallStr {
    #[inline(always)]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl core::ops::Deref for SmallStr {
    type Target = str;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl core::ops::DerefMut for SmallStr {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_str()
    }
}

impl core::borrow::Borrow<str> for SmallStr {
    #[inline(always)]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

// AsRef impls

impl AsRef<std::path::Path> for SmallStr {
    #[inline(always)]
    fn as_ref(&self) -> &std::path::Path {
        self.as_str().as_ref()
    }
}

impl AsRef<[u8]> for SmallStr {
    #[inline(always)]
    fn as_ref(&self) -> &[u8] {
        self.as_str().as_bytes()
    }
}

impl AsRef<std::ffi::OsStr> for SmallStr {
    #[inline(always)]
    fn as_ref(&self) -> &std::ffi::OsStr {
        self.as_str().as_ref()
    }
}

// From impls

impl From<&str> for SmallStr {
    #[inline(always)]
    fn from(value: &str) -> Self {
        Self(Inner::from(value))
    }
}

impl From<&&str> for SmallStr {
    #[inline(always)]
    fn from(value: &&str) -> Self {
        Self(Inner::from(*value))
    }
}

impl From<String> for SmallStr {
    #[inline(always)]
    fn from(value: String) -> Self {
        Self::from_string(value)
    }
}

impl From<SmallStr> for String {
    #[inline(always)]
    fn from(value: SmallStr) -> Self {
        value.to_string()
    }
}

impl From<Cow<'_, str>> for SmallStr {
    #[inline(always)]
    fn from(value: Cow<str>) -> Self {
        Self(Inner::from(value))
    }
}

impl From<&String> for SmallStr {
    #[inline(always)]
    fn from(value: &String) -> Self {
        Self(Inner::from(value.as_str()))
    }
}

impl From<Inner> for SmallStr {
    #[inline(always)]
    fn from(value: Inner) -> Self {
        Self(value)
    }
}

// FromIterator impls

impl FromIterator<SmallStr> for SmallStr {
    #[inline(always)]
    fn from_iter<T: IntoIterator<Item = SmallStr>>(iter: T) -> Self {
        Self(Inner::from_iter(iter.into_iter().map(|x| x.0)))
    }
}

impl<'a> FromIterator<&'a SmallStr> for SmallStr {
    #[inline(always)]
    fn from_iter<T: IntoIterator<Item = &'a SmallStr>>(iter: T) -> Self {
        Self(Inner::from_iter(iter.into_iter().map(|x| x.as_str())))
    }
}

impl FromIterator<char> for SmallStr {
    #[inline(always)]
    fn from_iter<I: IntoIterator<Item = char>>(iter: I) -> SmallStr {
        Self(Inner::from_iter(iter))
    }
}

impl<'a> FromIterator<&'a char> for SmallStr {
    #[inline(always)]
    fn from_iter<I: IntoIterator<Item = &'a char>>(iter: I) -> SmallStr {
        Self(Inner::from_iter(iter))
    }
}

impl<'a> FromIterator<&'a str> for SmallStr {
    #[inline(always)]
    fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> SmallStr {
        Self(Inner::from_iter(iter))
    }
}

impl FromIterator<String> for SmallStr {
    #[inline(always)]
    fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> SmallStr {
        Self(Inner::from_iter(iter))
    }
}

impl FromIterator<Box<str>> for SmallStr {
    #[inline(always)]
    fn from_iter<I: IntoIterator<Item = Box<str>>>(iter: I) -> SmallStr {
        Self(Inner::from_iter(iter))
    }
}

impl<'a> FromIterator<std::borrow::Cow<'a, str>> for SmallStr {
    #[inline(always)]
    fn from_iter<I: IntoIterator<Item = std::borrow::Cow<'a, str>>>(iter: I) -> SmallStr {
        Self(Inner::from_iter(iter))
    }
}

// PartialEq impls

impl<T> PartialEq<T> for SmallStr
where
    T: AsRef<str> + ?Sized,
{
    #[inline(always)]
    fn eq(&self, other: &T) -> bool {
        self.as_str() == other.as_ref()
    }
}

impl PartialEq<SmallStr> for &str {
    #[inline(always)]
    fn eq(&self, other: &SmallStr) -> bool {
        *self == other.as_str()
    }
}

impl PartialEq<SmallStr> for String {
    #[inline(always)]
    fn eq(&self, other: &SmallStr) -> bool {
        self.as_str() == other.as_str()
    }
}

// Write

impl core::fmt::Write for SmallStr {
    #[inline(always)]
    fn write_char(&mut self, c: char) -> std::fmt::Result {
        self.0.write_char(c)
    }

    #[inline(always)]
    fn write_fmt(&mut self, args: std::fmt::Arguments<'_>) -> std::fmt::Result {
        self.0.write_fmt(args)
    }

    #[inline(always)]
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.0.write_str(s)
    }
}

// Debug, Display

impl core::fmt::Debug for SmallStr {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

impl core::fmt::Display for SmallStr {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}
