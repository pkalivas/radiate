use compact_str::CompactString;
use radiate::RadiateResult;
use serde::{Deserialize, Serialize};

#[derive(Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeZone {
    inner: CompactString,
}

impl TimeZone {
    pub const UTC: TimeZone = unsafe { TimeZone::from_static("UTC") };

    /// Construct from a static string.
    ///
    /// # Safety
    /// This does not perform any validation, the caller is responsible for
    /// ensuring they pass a valid timezone.
    #[inline(always)]
    pub const unsafe fn from_static(tz: &'static str) -> Self {
        Self {
            inner: CompactString::const_new(tz),
        }
    }

    /// # Safety
    /// This does not perform any validation, the caller is responsible for
    /// ensuring they pass a valid timezone.
    pub unsafe fn new_unchecked(zone_str: impl Into<String>) -> Self {
        Self {
            inner: CompactString::from(zone_str.into()),
        }
    }

    /// Converts timezones to canonical form.
    ///
    /// If the "timezones" feature is enabled, additionally performs validation and converts to
    /// Etc/GMT form where applicable.
    #[inline]
    pub fn opt_try_new(zone_str: Option<impl Into<CompactString>>) -> RadiateResult<Option<Self>> {
        Self::new_impl(zone_str.map(|x| x.into()))
    }

    fn new_impl(zone_str: Option<CompactString>) -> RadiateResult<Option<Self>> {
        if zone_str.as_deref() == Some("*") {
            return Ok(Some(Self {
                inner: CompactString::const_new("*"),
            }));
        }

        let mut canonical_tz = Self::_canonical_timezone_impl(zone_str);

        if let Some(tz) = canonical_tz.as_mut() {
            if Self::validate_time_zone(tz).is_err() {
                match parse_fixed_offset(tz) {
                    Ok(v) => *tz = v.into(),
                    Err(err) => {
                        return Err(
                            err.context(format!("Unable to parse time zone string '{}'", tz))
                        );
                    }
                }
            }
        }

        Ok(canonical_tz.map(|inner| Self { inner }))
    }

    /// Equality where `None` is treated as UTC.
    pub fn eq_none_as_utc(this: Option<&TimeZone>, other: Option<&TimeZone>) -> bool {
        this.unwrap_or(&Self::UTC) == other.unwrap_or(&Self::UTC)
    }

    pub fn _canonical_timezone_impl(tz: Option<CompactString>) -> Option<CompactString> {
        match tz.as_deref() {
            Some("") | None => None,
            Some("+00:00") | Some("00:00") | Some("utc") => Some(CompactString::const_new("UTC")),
            Some(_) => tz,
        }
    }

    pub fn from_chrono(tz: &chrono_tz::Tz) -> Self {
        Self {
            inner: CompactString::from(tz.name()),
        }
    }

    pub fn to_chrono(&self) -> RadiateResult<chrono_tz::Tz> {
        parse_time_zone(self)
    }

    pub fn validate_time_zone(tz: &str) -> RadiateResult<()> {
        parse_time_zone(tz).map(|_| ())
    }
}

impl std::ops::Deref for TimeZone {
    type Target = CompactString;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::fmt::Debug for TimeZone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.inner, f)
    }
}

impl std::fmt::Display for TimeZone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.inner, f)
    }
}

static FIXED_OFFSET_PATTERN: &str = r#"(?x)
    ^
    (?P<sign>[-+])?            # optional sign
    (?P<hour>0[0-9]|1[0-4])    # hour (between 0 and 14)
    :?                         # optional separator
    00                         # minute
    $
    "#;

radiate_utils::cached_regex! {
    static FIXED_OFFSET_RE = FIXED_OFFSET_PATTERN;
}

/// Parse a time zone string to [`chrono_tz::Tz`]
pub fn parse_time_zone(tz: &str) -> RadiateResult<chrono_tz::Tz> {
    match tz.parse::<chrono_tz::Tz>() {
        Ok(tz) => Ok(tz),
        Err(_) => unable_to_parse_err(tz),
    }
}

/// Convert fixed offset to Etc/GMT one from time zone database
///
/// E.g. +01:00 -> Etc/GMT-1
///
/// Note: the sign appears reversed, but is correct, see <https://en.wikipedia.org/wiki/Tz_database#Area>:
/// > In order to conform with the POSIX style, those zone names beginning with
/// > "Etc/GMT" have their sign reversed from the standard ISO 8601 convention.
/// > In the "Etc" area, zones west of GMT have a positive sign and those east
/// > have a negative sign in their name (e.g "Etc/GMT-14" is 14 hours ahead of GMT).
pub fn parse_fixed_offset(tz: &str) -> RadiateResult<String> {
    if let Some(caps) = FIXED_OFFSET_RE.captures(tz) {
        let sign = match caps.name("sign").map(|s| s.as_str()) {
            Some("-") => "+",
            _ => "-",
        };
        let hour = caps.name("hour").unwrap().as_str().parse::<i32>().unwrap();
        let etc_tz = format!("Etc/GMT{}{}", sign, hour);
        if etc_tz.parse::<chrono_tz::Tz>().is_ok() {
            return Ok(etc_tz);
        }
    }

    unable_to_parse_err(tz)
}

fn unable_to_parse_err<T>(tz: &str) -> RadiateResult<T> {
    use radiate_error::radiate_bail;

    radiate_bail!(format!(
        "Unable to parse time zone string {:?}. \
        Refer to the tz database for valid time zone names: \
        https://en.wikipedia.org/wiki/Tz_database#List_of_tz_database_time_zones",
        tz
    ))
}
