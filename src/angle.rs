// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Angle primitives and sexagesimal parsing/formatting.
//!
//! Provenance: [`Angle`], the sexagesimal tokenizer, and their tests are
//! extracted from the sibling crate `target-match`; the explicit
//! [`ParseMode`]/[`SexaStyle`] surface and hour normalization are new here.
//!
//! - [`Angle`] — a unit-aware angle (degrees / radians / arcminutes /
//!   arcseconds / hours) stored internally in radians.
//! - [`parse_ra`] / [`parse_dec`] — sexagesimal (or bare-decimal) parsing in
//!   [`ParseMode::Strict`] or [`ParseMode::Lenient`]. Lenient tolerates
//!   missing minutes/seconds and mixed separators; **corrupt tokens are an
//!   error in every mode** — no input is ever silently dropped.
//! - [`format_ra`] / [`format_dec`] — sexagesimal formatting with rounding
//!   carry (never emits `60` in a minutes or seconds field).
//! - [`circular_mean`] / [`circular_distance`] — circular statistics over
//!   full-turn angles (vector-sum mean, shortest-arc distance).
//!
//! ```
//! use skymath::{format_dec, format_ra, parse_dec, parse_ra, ParseMode, SexaStyle};
//!
//! // M31's canonical J2000 position.
//! let ra = parse_ra("00:42:44.3", ParseMode::Strict).unwrap();
//! let dec = parse_dec("+41:16:09", ParseMode::Strict).unwrap();
//! assert_eq!(format_ra(ra, SexaStyle::default()), "00:42:44.30");
//! assert_eq!(format_dec(dec, SexaStyle::default()), "+41:16:09.00");
//! ```

use core::f64::consts::PI;
use core::ops::{Add, Div, Mul, Neg, Sub};

use crate::error::{Error, Result};

const DEG_PER_RAD: f64 = 180.0 / PI;
const RAD_PER_DEG: f64 = PI / 180.0;
/// Exact number of arcseconds in one radian.
pub const ARCSEC_PER_RADIAN: f64 = 206_264.806_247_096_36;

// ── Angle ──────────────────────────────────────────────────────────────────────

/// A unit-aware angle, stored internally in radians.
///
/// Construction and read-out are available in degrees, radians, arcminutes,
/// arcseconds, and hours (1 hour = 15°). Normalization is explicit — an `Angle`
/// holds whatever finite value it was given until you ask for a normalized form.
///
/// ```
/// use skymath::Angle;
///
/// // M31's right ascension, 00:42:44.3, expressed in hours then read out in degrees.
/// let ra = Angle::from_hours(42.0 / 60.0 + 44.3 / 3600.0);
/// assert!((ra.degrees() - 10.685).abs() < 1e-3);
/// assert!((Angle::from_degrees(370.0).normalized_0_360().degrees() - 10.0).abs() < 1e-9);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Angle {
    radians: f64,
}

impl Angle {
    /// Construct from radians.
    ///
    /// ```
    /// use skymath::Angle;
    ///
    /// let right_angle = Angle::from_radians(std::f64::consts::FRAC_PI_2);
    /// assert!((right_angle.degrees() - 90.0).abs() < 1e-9);
    /// ```
    #[must_use]
    pub const fn from_radians(radians: f64) -> Self {
        Self { radians }
    }
    /// Construct from decimal degrees.
    ///
    /// ```
    /// use skymath::Angle;
    ///
    /// assert!((Angle::from_degrees(180.0).radians() - std::f64::consts::PI).abs() < 1e-12);
    /// ```
    #[must_use]
    pub fn from_degrees(degrees: f64) -> Self {
        Self {
            radians: degrees * RAD_PER_DEG,
        }
    }
    /// Construct from arcminutes (1/60 degree).
    ///
    /// ```
    /// use skymath::Angle;
    ///
    /// assert!((Angle::from_arcminutes(30.0).degrees() - 0.5).abs() < 1e-12);
    /// ```
    #[must_use]
    pub fn from_arcminutes(arcmin: f64) -> Self {
        Self::from_degrees(arcmin / 60.0)
    }
    /// Construct from arcseconds (1/3600 degree).
    ///
    /// ```
    /// use skymath::Angle;
    ///
    /// assert!((Angle::from_arcseconds(3600.0).degrees() - 1.0).abs() < 1e-9);
    /// ```
    #[must_use]
    pub fn from_arcseconds(arcsec: f64) -> Self {
        Self::from_degrees(arcsec / 3600.0)
    }
    /// Construct from hours of right ascension (1 hour = 15°).
    ///
    /// ```
    /// use skymath::Angle;
    ///
    /// assert!((Angle::from_hours(6.0).degrees() - 90.0).abs() < 1e-9);
    /// ```
    #[must_use]
    pub fn from_hours(hours: f64) -> Self {
        Self::from_degrees(hours * 15.0)
    }

    /// Value in radians.
    ///
    /// ```
    /// use skymath::Angle;
    ///
    /// assert!((Angle::from_degrees(90.0).radians() - std::f64::consts::FRAC_PI_2).abs() < 1e-12);
    /// ```
    #[must_use]
    pub const fn radians(self) -> f64 {
        self.radians
    }
    /// Value in decimal degrees.
    ///
    /// ```
    /// use skymath::Angle;
    ///
    /// assert!((Angle::from_radians(std::f64::consts::PI).degrees() - 180.0).abs() < 1e-9);
    /// ```
    #[must_use]
    pub fn degrees(self) -> f64 {
        self.radians * DEG_PER_RAD
    }
    /// Value in arcminutes.
    ///
    /// ```
    /// use skymath::Angle;
    ///
    /// assert!((Angle::from_degrees(1.0).arcminutes() - 60.0).abs() < 1e-9);
    /// ```
    #[must_use]
    pub fn arcminutes(self) -> f64 {
        self.degrees() * 60.0
    }
    /// Value in arcseconds.
    ///
    /// ```
    /// use skymath::Angle;
    ///
    /// assert!((Angle::from_degrees(1.0).arcseconds() - 3600.0).abs() < 1e-6);
    /// ```
    #[must_use]
    pub fn arcseconds(self) -> f64 {
        self.degrees() * 3600.0
    }
    /// Value in hours (degrees / 15).
    ///
    /// ```
    /// use skymath::Angle;
    ///
    /// assert!((Angle::from_degrees(90.0).hours() - 6.0).abs() < 1e-9);
    /// ```
    #[must_use]
    pub fn hours(self) -> f64 {
        self.degrees() / 15.0
    }

    /// Return an equivalent angle wrapped into `[0, 360)` degrees.
    ///
    /// ```
    /// use skymath::Angle;
    ///
    /// assert!((Angle::from_degrees(-10.0).normalized_0_360().degrees() - 350.0).abs() < 1e-9);
    /// ```
    #[must_use]
    pub fn normalized_0_360(self) -> Self {
        let mut d = self.degrees() % 360.0;
        if d < 0.0 {
            d += 360.0;
        }
        Self::from_degrees(d)
    }
    /// Return an equivalent angle wrapped into `(-180, 180]` degrees.
    ///
    /// ```
    /// use skymath::Angle;
    ///
    /// assert!((Angle::from_degrees(350.0).normalized_pm_180().degrees() + 10.0).abs() < 1e-9);
    /// ```
    #[must_use]
    pub fn normalized_pm_180(self) -> Self {
        let mut d = self.normalized_0_360().degrees();
        if d > 180.0 {
            d -= 360.0;
        }
        Self::from_degrees(d)
    }
    /// Return an equivalent angle wrapped into `[0, 24)` hours.
    ///
    /// ```
    /// use skymath::Angle;
    ///
    /// assert!((Angle::from_hours(25.0).normalized_hours().hours() - 1.0).abs() < 1e-9);
    /// ```
    #[must_use]
    pub fn normalized_hours(self) -> Self {
        self.normalized_0_360()
    }
}

impl Add for Angle {
    type Output = Angle;
    fn add(self, rhs: Angle) -> Angle {
        Angle::from_radians(self.radians + rhs.radians)
    }
}
impl Sub for Angle {
    type Output = Angle;
    fn sub(self, rhs: Angle) -> Angle {
        Angle::from_radians(self.radians - rhs.radians)
    }
}
impl Neg for Angle {
    type Output = Angle;
    fn neg(self) -> Angle {
        Angle::from_radians(-self.radians)
    }
}
impl Mul<f64> for Angle {
    type Output = Angle;
    fn mul(self, rhs: f64) -> Angle {
        Angle::from_radians(self.radians * rhs)
    }
}
impl Div<f64> for Angle {
    type Output = Angle;
    fn div(self, rhs: f64) -> Angle {
        Angle::from_radians(self.radians / rhs)
    }
}

// ── Circular statistics ────────────────────────────────────────────────────────

/// Circular (vector-sum) mean of a set of angles, normalized to
/// `[0°, 360°)`; `None` for an empty input.
///
/// Each angle contributes a unit vector; the mean is the direction of the
/// resultant, so wraparound is handled correctly — the mean of 359° and 1°
/// is 0°, not 180°. Useful for averaging right ascensions or camera rotation
/// angles, where an arithmetic mean is wrong across the 0°/360° seam.
///
/// Caution: a near-antipodal set (e.g. `[0°, 180°]`) has a near-zero
/// resultant, so while the result is always a finite angle, its direction is
/// numerically meaningless — floating-point residue decides it, not a
/// meaningful "center" of opposite points.
///
/// ```
/// use skymath::{circular_mean, Angle};
///
/// let mean = circular_mean([359.0, 1.0].map(Angle::from_degrees)).unwrap();
/// assert!(circular_mean(std::iter::empty()).is_none());
/// assert!(mean.normalized_pm_180().degrees().abs() < 1e-9);
/// ```
pub fn circular_mean(angles: impl IntoIterator<Item = Angle>) -> Option<Angle> {
    let mut acc = CircularMean::new();
    for a in angles {
        acc.push(a);
    }
    acc.mean()
}

/// Running circular-mean accumulator: [`circular_mean`] for angles that
/// arrive incrementally (e.g. while clustering), without buffering them.
///
/// Maintains running unit-vector sums, so the result is exact for the pushed
/// set — no incremental-update approximation — and (up to float rounding
/// order) independent of push order. The same antipodal caution as
/// [`circular_mean`] applies.
///
/// ```
/// use skymath::{Angle, CircularMean};
///
/// let mut acc = CircularMean::new();
/// assert!(acc.mean().is_none());
/// acc.push(Angle::from_degrees(359.0));
/// acc.push(Angle::from_degrees(1.0));
/// assert!(acc.mean().unwrap().normalized_pm_180().degrees().abs() < 1e-9);
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct CircularMean {
    sin_sum: f64,
    cos_sum: f64,
    count: u64,
}

impl CircularMean {
    /// An empty accumulator ([`mean`](Self::mean) is `None`).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// Fold one angle into the running mean.
    pub fn push(&mut self, angle: Angle) {
        self.sin_sum += angle.radians.sin();
        self.cos_sum += angle.radians.cos();
        self.count += 1;
    }
    /// Circular mean of everything pushed so far, normalized to
    /// `[0°, 360°)`; `None` when nothing has been pushed.
    #[must_use]
    pub fn mean(&self) -> Option<Angle> {
        (self.count > 0)
            .then(|| Angle::from_radians(self.sin_sum.atan2(self.cos_sum)).normalized_0_360())
    }
}

/// Shortest arc between two angles on the full circle, in `[0°, 180°]`.
///
/// Symmetric and wraparound-safe: 350° and 10° are 20° apart, not 340°.
/// Treats the full turn as the period — two directions 180° apart are
/// maximally distant. For axially symmetric quantities where θ and θ+180°
/// are equivalent (e.g. a rotator ignoring image parity), halve the period
/// yourself: `circular_distance(a * 2.0, b * 2.0) / 2.0`.
///
/// ```
/// use skymath::{circular_distance, Angle};
///
/// let d = circular_distance(Angle::from_degrees(350.0), Angle::from_degrees(10.0));
/// assert!((d.degrees() - 20.0).abs() < 1e-9);
/// ```
#[must_use]
pub fn circular_distance(a: Angle, b: Angle) -> Angle {
    let diff = (a.radians - b.radians).rem_euclid(2.0 * PI);
    Angle::from_radians(diff.min(2.0 * PI - diff))
}

// ── Parse modes and styles ─────────────────────────────────────────────────────

/// How permissive sexagesimal parsing is.
///
/// In **every** mode an unparseable token is [`Error::ParseCoord`]; lenient
/// means flexible *format*, never acceptance of corrupt input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ParseMode {
    /// All three fields present (`HH:MM:SS` / `±DD MM SS`), colon or space
    /// separated, minutes/seconds in `[0, 60)`.
    Strict,
    /// One to three fields; missing minutes/seconds default to zero; separators
    /// may be spaces, colons, or tabs; the sign comes from the leading token.
    /// Bare decimals are accepted.
    #[default]
    Lenient,
}

/// Separator used in formatted sexagesimal output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Separator {
    /// `HH:MM:SS.sss` (display convention).
    #[default]
    Colons,
    /// `HH MM SS.sss` (FITS keyword convention).
    Spaces,
    /// Unicode astronomy glyphs: `HHhMMmSSs` for right ascension,
    /// `±DD°MM′SS″` for declination. Negative declination uses U+2212 (minus
    /// sign), not the ASCII hyphen `-`.
    ///
    /// ```
    /// use skymath::{format_dec, format_ra, parse_dec, parse_ra, ParseMode, Separator, SexaStyle};
    ///
    /// let style = SexaStyle { separator: Separator::Unicode, seconds_places: 0 };
    /// let ra = parse_ra("00:42:44.3", ParseMode::Strict).unwrap();
    /// assert_eq!(format_ra(ra, style), "00h42m44s");
    /// let dec = parse_dec("-05:30:00", ParseMode::Strict).unwrap();
    /// assert_eq!(format_dec(dec, style), "\u{2212}05\u{b0}30\u{2032}00\u{2033}");
    /// ```
    Unicode,
}

/// Formatting control for sexagesimal output.
///
/// ```
/// use skymath::{Separator, SexaStyle};
///
/// let fits_style = SexaStyle { separator: Separator::Spaces, seconds_places: 0 };
/// assert_eq!(fits_style.seconds_places, 0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SexaStyle {
    /// Field separator.
    pub separator: Separator,
    /// Fractional digits on the seconds field.
    pub seconds_places: u8,
}

impl Default for SexaStyle {
    /// Colon-separated with 2 fractional seconds digits.
    ///
    /// ```
    /// use skymath::{Separator, SexaStyle};
    ///
    /// let style = SexaStyle::default();
    /// assert_eq!(style.separator, Separator::Colons);
    /// assert_eq!(style.seconds_places, 2);
    /// ```
    fn default() -> Self {
        Self {
            separator: Separator::Colons,
            seconds_places: 2,
        }
    }
}

// ── Parsing ────────────────────────────────────────────────────────────────────

/// Parse a right ascension string into an [`Angle`].
///
/// Sexagesimal input is in **hours** (`"06:00:00"` → 90°); a bare decimal is
/// in **degrees** (matching FITS `RA`/`OBJCTRA` conventions). The result is
/// not domain-checked — wrap/validate at the type that embeds it.
///
/// # Errors
/// [`Error::ParseCoord`] on malformed input (any mode).
///
/// ```
/// use skymath::{parse_ra, ParseMode};
///
/// let ra = parse_ra("00:42:44.3", ParseMode::Strict)?;
/// assert!((ra.hours() - 0.7123).abs() < 1e-3);
/// # Ok::<(), skymath::Error>(())
/// ```
pub fn parse_ra(s: &str, mode: ParseMode) -> Result<Angle> {
    if looks_sexagesimal(s) {
        Ok(Angle::from_hours(parse_sexagesimal(s, mode)?))
    } else {
        decimal_fallback(s, mode).map(Angle::from_degrees)
    }
}

/// Parse a declination (or latitude/longitude) string into an [`Angle`].
///
/// Both sexagesimal and bare-decimal input are in **degrees**. The sign is
/// taken from the leading field and preserved even for `-00 30 00`.
///
/// # Errors
/// [`Error::ParseCoord`] on malformed input (any mode).
///
/// ```
/// use skymath::{parse_dec, ParseMode};
///
/// let dec = parse_dec("+41:16:09", ParseMode::Strict)?;
/// assert!((dec.degrees() - 41.269_17).abs() < 1e-3);
/// # Ok::<(), skymath::Error>(())
/// ```
pub fn parse_dec(s: &str, mode: ParseMode) -> Result<Angle> {
    if looks_sexagesimal(s) {
        Ok(Angle::from_degrees(parse_sexagesimal(s, mode)?))
    } else {
        decimal_fallback(s, mode).map(Angle::from_degrees)
    }
}

fn decimal_fallback(s: &str, mode: ParseMode) -> Result<f64> {
    match mode {
        // A single bare field is only acceptable in lenient mode.
        ParseMode::Strict => Err(Error::ParseCoord(format!(
            "strict mode requires HH:MM:SS / DD:MM:SS fields: {s:?}"
        ))),
        ParseMode::Lenient => parse_decimal(s),
    }
}

fn looks_sexagesimal(raw: &str) -> bool {
    let t = raw.trim();
    t.contains(':') || t.split_whitespace().count() > 1
}

fn parse_decimal(raw: &str) -> Result<f64> {
    raw.trim()
        .parse::<f64>()
        .ok()
        .filter(|v| v.is_finite())
        .ok_or_else(|| Error::ParseCoord(format!("not a finite number: {raw:?}")))
}

/// Parse `±A:B:C(.c)` / `±A B C` into a signed decimal value (degrees for Dec,
/// hours for RA). Minutes/seconds must be in `[0, 60)`; the sign comes from the
/// leading field; every token must parse — corrupt tokens are never dropped.
fn parse_sexagesimal(raw: &str, mode: ParseMode) -> Result<f64> {
    let trimmed = raw.trim().trim_matches('\'').trim();
    if trimmed.is_empty() {
        return Err(Error::ParseCoord("empty coordinate".to_owned()));
    }
    let normalized = trimmed.replace([':', '\t'], " ");
    let mut parts = normalized.split_whitespace();
    let lead = parts
        .next()
        .ok_or_else(|| Error::ParseCoord(format!("no leading field: {raw:?}")))?;
    let negative = lead.starts_with('-');
    let lead_val: f64 = lead
        .parse()
        .ok()
        .filter(|v: &f64| v.is_finite())
        .ok_or_else(|| Error::ParseCoord(format!("bad degrees/hours field: {raw:?}")))?;
    let min = next_field(&mut parts, raw, "minutes")?;
    let sec = next_field(&mut parts, raw, "seconds")?;
    if parts.next().is_some() {
        return Err(Error::ParseCoord(format!("too many fields: {raw:?}")));
    }
    let field_count = 1 + usize::from(min.is_some()) + usize::from(sec.is_some());
    if mode == ParseMode::Strict && field_count != 3 {
        return Err(Error::ParseCoord(format!(
            "strict mode requires 3 fields, got {field_count}: {raw:?}"
        )));
    }
    let (min, sec) = (min.unwrap_or(0.0), sec.unwrap_or(0.0));
    if min < 0.0 || sec < 0.0 || min >= 60.0 || sec >= 60.0 {
        return Err(Error::ParseCoord(format!(
            "minutes/seconds out of range: {raw:?}"
        )));
    }
    let magnitude = lead_val.abs() + min / 60.0 + sec / 3600.0;
    Ok(if negative { -magnitude } else { magnitude })
}

fn next_field<'a>(
    parts: &mut impl Iterator<Item = &'a str>,
    raw: &str,
    what: &str,
) -> Result<Option<f64>> {
    match parts.next() {
        None => Ok(None),
        Some(s) => s
            .parse::<f64>()
            .ok()
            .filter(|v| v.is_finite())
            .map(Some)
            .ok_or_else(|| Error::ParseCoord(format!("bad {what} field: {raw:?}"))),
    }
}

// ── Formatting ─────────────────────────────────────────────────────────────────

/// Format an angle as sexagesimal right ascension (hours), wrapped to
/// `[0h, 24h)`, e.g. `06:30:00.00`.
///
/// ```
/// use skymath::{format_ra, parse_ra, ParseMode, SexaStyle};
///
/// let ra = parse_ra("00:42:44.3", ParseMode::Strict)?;
/// assert_eq!(format_ra(ra, SexaStyle::default()), "00:42:44.30");
/// # Ok::<(), skymath::Error>(())
/// ```
#[must_use]
pub fn format_ra(a: Angle, style: SexaStyle) -> String {
    let hours = a.normalized_0_360().degrees() / 15.0;
    let s = format_sexagesimal(hours, false, style, true);
    // Rounding carry can reach 24:00:00 — wrap to 00.
    if s.starts_with("24:") || s.starts_with("24 ") || s.starts_with("24h") {
        format_sexagesimal(0.0, false, style, true)
    } else {
        s
    }
}

/// Format an angle as signed sexagesimal degrees (declination/latitude), e.g.
/// `+41:16:09.00`. The sign is always present; `-0°` keeps its minus sign.
///
/// ```
/// use skymath::{format_dec, parse_dec, ParseMode, SexaStyle};
///
/// let dec = parse_dec("+41:16:09", ParseMode::Strict)?;
/// assert_eq!(format_dec(dec, SexaStyle::default()), "+41:16:09.00");
/// # Ok::<(), skymath::Error>(())
/// ```
#[must_use]
pub fn format_dec(a: Angle, style: SexaStyle) -> String {
    format_sexagesimal(a.degrees(), true, style, false)
}

/// Format a signed decimal value as sexagesimal, with rounding performed at the
/// seconds precision *before* field splitting so `59.9996″` carries into the
/// minute (never emitting a `60` field). `is_ra` selects the RA (`h`/`m`/`s`)
/// vs Dec (`°`/`′`/`″`) glyph set when `style.separator` is
/// [`Separator::Unicode`].
fn format_sexagesimal(value: f64, signed: bool, style: SexaStyle, is_ra: bool) -> String {
    let neg = value.is_sign_negative() && value != 0.0;
    let decimals = usize::from(style.seconds_places);
    let sec_scale = 3600.0 * 10f64.powi(i32::from(style.seconds_places));
    let rounded = (value.abs() * sec_scale).round() / sec_scale;
    let (a, b, c) = decompose_magnitude(rounded);
    let width = if decimals > 0 { decimals + 3 } else { 2 };
    match style.separator {
        Separator::Colons | Separator::Spaces => {
            let sign = if neg {
                "-"
            } else if signed {
                "+"
            } else {
                ""
            };
            let sep = match style.separator {
                Separator::Colons => ':',
                Separator::Spaces => ' ',
                Separator::Unicode => unreachable!(),
            };
            format!("{sign}{a:02}{sep}{b:02}{sep}{c:0width$.decimals$}")
        }
        Separator::Unicode => {
            let sign = if neg {
                "\u{2212}"
            } else if signed {
                "+"
            } else {
                ""
            };
            let (u1, u2, u3) = if is_ra {
                ("h", "m", "s")
            } else {
                ("\u{b0}", "\u{2032}", "\u{2033}")
            };
            format!("{sign}{a:02}{u1}{b:02}{u2}{c:0width$.decimals$}{u3}")
        }
    }
}

/// Split a non-negative decimal value into `(whole, minutes, seconds)`
/// sexagesimal components. Shared by `format_sexagesimal` (after
/// display-precision rounding) and the raw component accessors on
/// `Equatorial` (no rounding there — callers own their own display
/// precision).
pub(crate) fn decompose_magnitude(v: f64) -> (u32, u32, f64) {
    let a = v.trunc();
    let rem_min = (v - a) * 60.0;
    let b = rem_min.trunc();
    let c = (rem_min - b) * 60.0;
    (a as u32, b as u32, c)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64, eps: f64) -> bool {
        (a - b).abs() < eps
    }

    #[test]
    fn angle_conversions() {
        let a = Angle::from_degrees(90.0);
        assert!(approx(a.radians(), PI / 2.0, 1e-12));
        assert!(approx(a.arcminutes(), 5400.0, 1e-6));
        assert!(approx(a.arcseconds(), 324_000.0, 1e-3));
        assert!(approx(Angle::from_hours(1.0).degrees(), 15.0, 1e-12));
        assert!(approx(Angle::from_arcminutes(60.0).degrees(), 1.0, 1e-12));
        assert!(approx(Angle::from_arcseconds(3600.0).degrees(), 1.0, 1e-12));
    }

    #[test]
    fn angle_normalization() {
        assert!(approx(
            Angle::from_degrees(370.0).normalized_0_360().degrees(),
            10.0,
            1e-9
        ));
        assert!(approx(
            Angle::from_degrees(-10.0).normalized_0_360().degrees(),
            350.0,
            1e-9
        ));
        assert!(approx(
            Angle::from_degrees(350.0).normalized_pm_180().degrees(),
            -10.0,
            1e-9
        ));
        assert!(approx(
            Angle::from_hours(25.0).normalized_hours().hours(),
            1.0,
            1e-9
        ));
    }

    #[test]
    fn angle_ops() {
        let s = Angle::from_degrees(10.0) + Angle::from_degrees(5.0);
        assert!(approx(s.degrees(), 15.0, 1e-12));
        assert!(approx(
            (Angle::from_degrees(10.0) * 3.0).degrees(),
            30.0,
            1e-12
        ));
        assert!(approx(
            (Angle::from_degrees(10.0) / 2.0).degrees(),
            5.0,
            1e-12
        ));
        assert!(approx((-Angle::from_degrees(10.0)).degrees(), -10.0, 1e-12));
    }

    #[test]
    fn lenient_accepts_partial_fields() {
        assert!(approx(
            parse_ra("10 30", ParseMode::Lenient).unwrap().hours(),
            10.5,
            1e-9
        ));
        assert!(approx(
            parse_dec("45", ParseMode::Lenient).unwrap().degrees(),
            45.0,
            1e-9
        ));
    }

    #[test]
    fn garbage_errors_in_every_mode() {
        for mode in [ParseMode::Strict, ParseMode::Lenient] {
            assert!(matches!(
                parse_ra("10 xx 30", mode).unwrap_err(),
                Error::ParseCoord(_)
            ));
            assert!(matches!(
                parse_dec("", mode).unwrap_err(),
                Error::ParseCoord(_)
            ));
            assert!(matches!(
                parse_dec("12 70 00", mode).unwrap_err(),
                Error::ParseCoord(_)
            ));
        }
    }

    #[test]
    fn strict_requires_three_fields() {
        assert!(parse_ra("10 30", ParseMode::Strict).is_err());
        assert!(parse_ra("10.5", ParseMode::Strict).is_err());
        assert!(parse_ra("10:30:00", ParseMode::Strict).is_ok());
    }

    #[test]
    fn sign_survives_zero_degrees() {
        let d = parse_dec("-00 30 00", ParseMode::Lenient).unwrap();
        assert!(approx(d.degrees(), -0.5, 1e-9));
        let formatted = format_dec(d, SexaStyle::default());
        assert!(formatted.starts_with("-00:30"), "{formatted}");
    }

    #[test]
    fn format_carries_rounding() {
        // 59.9996″ at 2 decimals must roll into the next minute, never ":60".
        let a = Angle::from_degrees(10.0 + 59.0 / 60.0 + 59.9996 / 3600.0);
        let s = format_dec(a, SexaStyle::default());
        assert_eq!(s, "+11:00:00.00");
        // RA carry across 24h wraps to zero.
        let ra = Angle::from_hours(23.0 + 59.0 / 60.0 + 59.9996 / 3600.0);
        let r = format_ra(ra, SexaStyle::default());
        assert_eq!(r, "00:00:00.00");
    }

    #[test]
    fn format_unicode_glyphs() {
        let style = SexaStyle {
            separator: Separator::Unicode,
            seconds_places: 0,
        };
        let ra = Angle::from_hours(0.712_306);
        assert_eq!(format_ra(ra, style), "00h42m44s");
        let dec = Angle::from_degrees(41.269_167);
        assert_eq!(format_dec(dec, style), "+41\u{b0}16\u{2032}09\u{2033}");
        let neg_dec = Angle::from_degrees(-5.5);
        assert_eq!(
            format_dec(neg_dec, style),
            "\u{2212}05\u{b0}30\u{2032}00\u{2033}"
        );
        assert!(
            !format_dec(neg_dec, style).starts_with('-'),
            "must use U+2212, not ASCII hyphen"
        );
    }

    #[test]
    fn format_unicode_ra_carries_across_24h() {
        let style = SexaStyle {
            separator: Separator::Unicode,
            seconds_places: 2,
        };
        let ra = Angle::from_hours(23.0 + 59.0 / 60.0 + 59.9996 / 3600.0);
        assert_eq!(format_ra(ra, style), "00h00m00.00s");
    }

    #[test]
    fn format_styles() {
        let a = Angle::from_hours(6.5);
        assert_eq!(
            format_ra(
                a,
                SexaStyle {
                    separator: Separator::Spaces,
                    seconds_places: 0
                }
            ),
            "06 30 00"
        );
        assert_eq!(format_ra(a, SexaStyle::default()), "06:30:00.00");
    }

    #[test]
    fn circular_mean_handles_wraparound() {
        let mean = circular_mean([359.0, 1.0].map(Angle::from_degrees)).unwrap();
        // 0° and 360° are the same direction; compare on the signed branch.
        assert!(mean.normalized_pm_180().degrees().abs() < 1e-9);
        let mean = circular_mean([10.0, 20.0, 30.0].map(Angle::from_degrees)).unwrap();
        assert!((mean.degrees() - 20.0).abs() < 1e-9);
        // Result is normalized even for out-of-range inputs.
        let mean = circular_mean([Angle::from_degrees(-90.0)]).unwrap();
        assert!((mean.degrees() - 270.0).abs() < 1e-9);
    }

    #[test]
    fn circular_mean_empty_is_none() {
        assert!(circular_mean(std::iter::empty()).is_none());
    }

    #[test]
    fn circular_mean_antipodal_is_degenerate_but_finite() {
        // Cancelling resultant: direction is float residue, but never NaN
        // and always normalized.
        let mean = circular_mean([0.0, 180.0].map(Angle::from_degrees)).unwrap();
        let deg = mean.degrees();
        assert!(deg.is_finite());
        assert!((0.0..360.0).contains(&deg));
    }

    #[test]
    fn circular_mean_accumulator_matches_batch() {
        let degs = [340.0, 355.0, 5.0, 20.0, 180.0];
        let batch = circular_mean(degs.map(Angle::from_degrees)).unwrap();
        let mut acc = CircularMean::new();
        for d in degs {
            acc.push(Angle::from_degrees(d));
        }
        assert!((acc.mean().unwrap().degrees() - batch.degrees()).abs() < 1e-12);

        // Push order only reorders float additions; results agree tightly.
        let mut rev = CircularMean::new();
        for d in degs.iter().rev() {
            rev.push(Angle::from_degrees(*d));
        }
        assert!((rev.mean().unwrap().degrees() - batch.degrees()).abs() < 1e-9);
    }

    #[test]
    fn circular_mean_accumulator_empty_is_none() {
        assert!(CircularMean::new().mean().is_none());
        assert!(CircularMean::default().mean().is_none());
    }

    #[test]
    fn circular_distance_shortest_arc() {
        let d = |a: f64, b: f64| {
            circular_distance(Angle::from_degrees(a), Angle::from_degrees(b)).degrees()
        };
        assert!((d(350.0, 10.0) - 20.0).abs() < 1e-9); // across the seam
        assert!((d(10.0, 350.0) - 20.0).abs() < 1e-9); // symmetric
        assert!((d(0.0, 180.0) - 180.0).abs() < 1e-9); // antipodal maximum
        assert!(d(42.0, 42.0).abs() < 1e-9);
        assert!((d(-10.0, 10.0) - 20.0).abs() < 1e-9); // unnormalized inputs
    }
}
