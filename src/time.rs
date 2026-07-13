//! Time scales: Julian dates, FITS `DATE-OBS` strings, Julian epochs, and
//! sidereal time.
//!
//! Provenance: [`parse_date_obs`] / [`format_date_obs`] are extracted from
//! `fits-header` (`src/dates.rs`); the MJD/JD conversions and [`gmst`] /
//! [`lst`] are written fresh, validated against Meeus, *Astronomical
//! Algorithms* 2nd ed. (chapters 7 and 12).
//!
//! GMST uses the IAU-1982 polynomial (Meeus eq. 12.4) on the UT1 ≈ UTC
//! assumption: |ΔUT1| < 0.9 s of time ≈ 13.5″ of hour angle, inside the
//! crate's planning-grade contract. Functions taking [`OffsetDateTime`]
//! convert to UTC internally, so passing local civil time cannot skew
//! sidereal results. JD/MJD are carried as `f64` — microsecond-level
//! resolution in the current era.

use ::time::{Date, Month, OffsetDateTime, PrimitiveDateTime, Time};

use crate::angle::Angle;
use crate::coords::Epoch;
use crate::error::{Error, Result};

/// JD − MJD: the MJD epoch (1858-11-17T00:00 UTC) as a Julian date.
const MJD_JD_OFFSET: f64 = 2_400_000.5;
/// Julian day number of the MJD epoch's calendar day (`to_julian_day` of
/// 1858-11-17, i.e. the JD of that day's *noon*).
const MJD_EPOCH_JDN: i64 = 2_400_001;
/// J2000.0 (2000-01-01T12:00 TT ≈ UTC at planning grade) as a Julian date.
const J2000_JD: f64 = 2_451_545.0;
const SECONDS_PER_DAY: f64 = 86_400.0;
const NANOS_PER_DAY: i64 = 86_400 * 1_000_000_000;

// ── Julian / Modified Julian dates ─────────────────────────────────────────────

/// Modified Julian Date of a timezone-naive instant (taken as UTC). Inverse
/// of [`mjd_to_datetime`].
///
/// ```
/// use skymath::datetime_to_mjd;
/// use time::macros::datetime;
///
/// // J2000.0 = MJD 51544.5.
/// assert_eq!(datetime_to_mjd(datetime!(2000-01-01 12:00)), 51_544.5);
/// ```
pub fn datetime_to_mjd(dt: PrimitiveDateTime) -> f64 {
    let days = (i64::from(dt.date().to_julian_day()) - MJD_EPOCH_JDN) as f64;
    let t = dt.time();
    let day_seconds = f64::from(t.hour()) * 3_600.0
        + f64::from(t.minute()) * 60.0
        + f64::from(t.second())
        + f64::from(t.nanosecond()) / 1e9;
    days + day_seconds / SECONDS_PER_DAY
}

/// Timezone-naive UTC instant of a Modified Julian Date. Inverse of
/// [`datetime_to_mjd`].
///
/// Errors with [`Error::OutOfRange`] when `mjd` is not finite or falls
/// outside the `time` crate's representable calendar range.
///
/// ```
/// use skymath::mjd_to_datetime;
/// use time::macros::datetime;
///
/// assert_eq!(mjd_to_datetime(51_544.5).unwrap(), datetime!(2000-01-01 12:00));
/// assert!(mjd_to_datetime(f64::NAN).is_err());
/// ```
pub fn mjd_to_datetime(mjd: f64) -> Result<PrimitiveDateTime> {
    let out_of_range = || Error::OutOfRange {
        what: "mjd",
        value: mjd,
    };
    if !mjd.is_finite() {
        return Err(out_of_range());
    }

    let day = mjd.floor();
    let mut jdn = day as i64 + MJD_EPOCH_JDN;
    let mut nanos = ((mjd - day) * SECONDS_PER_DAY * 1e9).round() as i64;
    if nanos >= NANOS_PER_DAY {
        // The fractional part rounded up to a full day.
        nanos -= NANOS_PER_DAY;
        jdn += 1;
    }

    let date = i32::try_from(jdn)
        .ok()
        .and_then(|j| Date::from_julian_day(j).ok())
        .ok_or_else(out_of_range)?;
    let hour = (nanos / 3_600_000_000_000) as u8;
    let minute = ((nanos / 60_000_000_000) % 60) as u8;
    let second = ((nanos / 1_000_000_000) % 60) as u8;
    let nano = (nanos % 1_000_000_000) as u32;
    let time = Time::from_hms_nano(hour, minute, second, nano)
        .expect("components are in range by construction");
    Ok(PrimitiveDateTime::new(date, time))
}

/// Convert a Julian Date to a Modified Julian Date. Inverse of [`mjd_to_jd`].
///
/// ```
/// use skymath::jd_to_mjd;
///
/// assert_eq!(jd_to_mjd(2_451_545.0), 51_544.5); // J2000.0
/// ```
pub fn jd_to_mjd(jd: f64) -> f64 {
    jd - MJD_JD_OFFSET
}

/// Convert a Modified Julian Date to a Julian Date. Inverse of [`jd_to_mjd`].
///
/// ```
/// use skymath::mjd_to_jd;
///
/// assert_eq!(mjd_to_jd(51_544.5), 2_451_545.0); // J2000.0
/// ```
pub fn mjd_to_jd(mjd: f64) -> f64 {
    mjd + MJD_JD_OFFSET
}

/// Julian Date of an instant; the offset is folded in (UTC internally).
///
/// ```
/// use skymath::julian_date;
/// use time::macros::datetime;
///
/// assert_eq!(julian_date(datetime!(2000-01-01 12:00 UTC)), 2_451_545.0);
/// ```
pub fn julian_date(at: OffsetDateTime) -> f64 {
    // The Unix epoch (1970-01-01T00:00 UTC) is JD 2 440 587.5.
    2_440_587.5 + at.unix_timestamp_nanos() as f64 / (SECONDS_PER_DAY * 1e9)
}

// ── FITS DATE-OBS ──────────────────────────────────────────────────────────────

/// Parse a FITS civil date/time (`YYYY-MM-DD[Thh:mm:ss[.fff]]`),
/// timezone-naive; a date-only value means midnight. Quoted card values and
/// surrounding whitespace are tolerated. FITS strings carry no offset — bridge
/// to [`OffsetDateTime`] with `parse_date_obs(s)?.assume_utc()`. Inverse of
/// [`format_date_obs`].
///
/// ```
/// use skymath::parse_date_obs;
/// use time::macros::datetime;
///
/// assert_eq!(parse_date_obs("2026-07-11T22:15:03.25")?, datetime!(2026-07-11 22:15:03.25));
/// assert_eq!(parse_date_obs("2026-07-11")?, datetime!(2026-07-11 00:00));
/// # Ok::<(), skymath::Error>(())
/// ```
pub fn parse_date_obs(s: &str) -> Result<PrimitiveDateTime> {
    parse_date_obs_opt(s).ok_or_else(|| Error::ParseDate(s.trim().to_string()))
}

fn parse_date_obs_opt(s: &str) -> Option<PrimitiveDateTime> {
    let t = s.trim().trim_matches('\'').trim();
    let (date_part, time_part) = match t.split_once('T') {
        Some((d, tm)) => (d, Some(tm)),
        None => (t, None),
    };

    let mut d = date_part.split('-');
    let year: i32 = d.next()?.parse().ok()?;
    let month: u8 = d.next()?.parse().ok()?;
    let day: u8 = d.next()?.parse().ok()?;
    if d.next().is_some() {
        return None;
    }
    let date = Date::from_calendar_date(year, Month::try_from(month).ok()?, day).ok()?;

    let time = match time_part {
        None => Time::MIDNIGHT,
        Some(tp) => {
            let mut parts = tp.split(':');
            let hour: u8 = parts.next()?.parse().ok()?;
            let minute: u8 = parts.next()?.parse().ok()?;
            let (sec, nanos) = match parts.next() {
                None => (0u8, 0u32),
                Some(sec_field) => {
                    let (whole, frac) = match sec_field.split_once('.') {
                        Some((w, f)) => (w, Some(f)),
                        None => (sec_field, None),
                    };
                    let sec: u8 = whole.parse().ok()?;
                    let nanos = match frac {
                        None => 0,
                        Some(f) => {
                            let mut digits: String = f.chars().take(9).collect();
                            while digits.len() < 9 {
                                digits.push('0');
                            }
                            digits.parse::<u32>().ok()?
                        }
                    };
                    (sec, nanos)
                }
            };
            if parts.next().is_some() {
                return None;
            }
            Time::from_hms_nano(hour, minute, sec, nanos).ok()?
        }
    };

    Some(PrimitiveDateTime::new(date, time))
}

/// Format a date/time back to the FITS civil form
/// (`YYYY-MM-DDThh:mm:ss[.fff]`), dropping a zero sub-second part. Inverse of
/// [`parse_date_obs`].
///
/// ```
/// use skymath::format_date_obs;
/// use time::macros::datetime;
///
/// assert_eq!(format_date_obs(datetime!(2026-07-11 22:15:00)), "2026-07-11T22:15:00");
/// ```
pub fn format_date_obs(dt: PrimitiveDateTime) -> String {
    let (d, t) = (dt.date(), dt.time());
    let base = format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
        d.year(),
        d.month() as u8,
        d.day(),
        t.hour(),
        t.minute(),
        t.second()
    );
    let nanos = t.nanosecond();
    if nanos == 0 {
        base
    } else {
        let frac = format!("{nanos:09}");
        format!("{base}.{}", frac.trim_end_matches('0'))
    }
}

// ── Julian epoch & sidereal time ───────────────────────────────────────────────

/// The Julian epoch of an instant, e.g. `Epoch::OfDate(2026.52…)` for
/// mid-July 2026. Feeds [`precess`](crate::precess) when moving an
/// [`Equatorial`](crate::Equatorial) position to "tonight".
///
/// ```
/// use skymath::julian_epoch_of;
/// use skymath::Epoch;
/// use time::macros::datetime;
///
/// assert_eq!(
///     julian_epoch_of(datetime!(2000-01-01 12:00 UTC)),
///     Epoch::OfDate(2000.0)
/// );
/// ```
pub fn julian_epoch_of(at: OffsetDateTime) -> Epoch {
    Epoch::OfDate(2_000.0 + (julian_date(at) - J2000_JD) / 365.25)
}

/// Greenwich Mean Sidereal Time, normalized to `[0h, 24h)`. Feeds [`lst`] at
/// an observer's longitude.
///
/// IAU-1982 polynomial (Meeus eq. 12.4) on UT1 ≈ UTC: accurate to ~0.1 s of
/// time over ±1 century around J2000.
///
/// ```
/// use skymath::gmst;
/// use time::OffsetDateTime;
///
/// let hours = gmst(OffsetDateTime::now_utc()).hours();
/// assert!((0.0..24.0).contains(&hours));
/// ```
pub fn gmst(at: OffsetDateTime) -> Angle {
    let d = julian_date(at) - J2000_JD;
    let t = d / 36_525.0;
    let degrees =
        280.460_618_37 + 360.985_647_366_29 * d + 0.000_387_933 * t * t - t * t * t / 38_710_000.0;
    Angle::from_degrees(degrees).normalized_hours()
}

/// Local Sidereal Time at an east-positive longitude, normalized to
/// `[0h, 24h)`. `lst(at, lon) == gmst(at) + lon` (wrapped).
///
/// ```
/// use skymath::{gmst, lst, Location};
/// use time::OffsetDateTime;
///
/// let site = Location::parse("+52 05 32", "+004 18 27", 6.0)?;
/// let now = OffsetDateTime::now_utc();
/// let expected = (gmst(now) + site.longitude()).normalized_hours();
/// assert!((lst(now, site.longitude()).hours() - expected.hours()).abs() < 1e-9);
/// # Ok::<(), skymath::Error>(())
/// ```
pub fn lst(at: OffsetDateTime, longitude_east: Angle) -> Angle {
    (gmst(at) + longitude_east).normalized_hours()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::time::macros::datetime;

    // Extracted with parse/format from fits-header `src/dates.rs`.
    #[test]
    fn date_only_is_midnight() {
        let dt = parse_date_obs("2026-07-11").unwrap();
        assert_eq!(dt.time(), Time::MIDNIGHT);
        assert_eq!(format_date_obs(dt), "2026-07-11T00:00:00");
    }

    #[test]
    fn seconds_and_fraction_are_optional() {
        assert_eq!(
            format_date_obs(parse_date_obs("2026-07-11T22:15").unwrap()),
            "2026-07-11T22:15:00"
        );
        let dt = parse_date_obs("2026-07-11T22:15:03.25").unwrap();
        assert_eq!(dt.time().nanosecond(), 250_000_000);
        assert_eq!(format_date_obs(dt), "2026-07-11T22:15:03.25");
    }

    #[test]
    fn fraction_beyond_nanoseconds_is_truncated() {
        let dt = parse_date_obs("2026-07-11T00:00:00.1234567891234").unwrap();
        assert_eq!(dt.time().nanosecond(), 123_456_789);
    }

    #[test]
    fn quoted_input_is_tolerated() {
        assert!(parse_date_obs("'2026-07-11T01:02:03'").is_ok());
        assert!(parse_date_obs("  2026-07-11  ").is_ok());
    }

    #[test]
    fn invalid_forms_error_with_input() {
        for bad in [
            "2026-13-01",          // month
            "2026-02-30",          // day
            "2026-07-11T25:00:00", // hour
            "2026-07-11-05",       // extra date part
            "2026-07-11T1:2:3:4",  // extra time part
            "2026",                // no month/day
            "not a date",
        ] {
            assert_eq!(
                parse_date_obs(bad),
                Err(Error::ParseDate(bad.to_string())),
                "{bad:?} should not parse"
            );
        }
    }

    #[test]
    fn format_trims_trailing_fraction_zeros() {
        let dt = parse_date_obs("2026-07-11T00:00:00.100").unwrap();
        assert_eq!(format_date_obs(dt), "2026-07-11T00:00:00.1");
        let dt = parse_date_obs("2026-07-11T00:00:00.000").unwrap();
        assert_eq!(format_date_obs(dt), "2026-07-11T00:00:00");
    }

    #[test]
    fn mjd_anchors() {
        // The MJD epoch itself.
        assert_eq!(datetime_to_mjd(datetime!(1858-11-17 00:00)), 0.0);
        // J2000.0 = JD 2 451 545.0 = MJD 51 544.5.
        let j2000 = datetime!(2000-01-01 12:00);
        assert_eq!(datetime_to_mjd(j2000), 51_544.5);
        assert_eq!(mjd_to_jd(51_544.5), 2_451_545.0);
        assert_eq!(jd_to_mjd(2_451_545.0), 51_544.5);
        assert_eq!(julian_date(j2000.assume_utc()), 2_451_545.0);
    }

    #[test]
    fn mjd_round_trips_through_datetime() {
        let dt = datetime!(2026-07-11 22:15:03.25);
        let back = mjd_to_datetime(datetime_to_mjd(dt)).unwrap();
        let delta = (back - dt).abs();
        assert!(delta < ::time::Duration::microseconds(5), "delta {delta}");
    }

    #[test]
    fn mjd_fraction_rounding_up_carries_to_next_day() {
        // One sub-nanosecond step below a whole day must not produce 24:00:00.
        let mjd = 60_964.999_999_999_999_9;
        let dt = mjd_to_datetime(mjd).unwrap();
        assert_eq!(dt.time(), Time::MIDNIGHT);
    }

    #[test]
    fn mjd_rejects_unrepresentable() {
        assert!(mjd_to_datetime(f64::NAN).is_err());
        assert!(mjd_to_datetime(f64::INFINITY).is_err());
        assert!(mjd_to_datetime(1e18).is_err());
    }

    #[test]
    fn julian_date_folds_in_the_offset() {
        let utc = datetime!(2000-01-01 12:00 UTC);
        let dubai = datetime!(2000-01-01 16:00 +04:00);
        assert_eq!(julian_date(utc), julian_date(dubai));
    }
}
