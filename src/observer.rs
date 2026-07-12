//! Observer-local quantities: site location, hour angle, alt-azimuth,
//! airmass, refraction, parallactic angle, and transit/altitude crossings.
//!
//! Provenance: the alt-azimuth spherical-triangle formulation (with its
//! zenith/pole degeneracy handling) and the AstroPy cross-check vectors in
//! `tests/ported_vectors.rs` are ported from `gaker/astro-math`
//! (`src/transforms.rs`, see `NOTICE`). [`Location::parse`] reuses skymath's
//! lenient sexagesimal parser — a deliberate trim of astro-math's regex-based
//! location parser. Airmass (Kasten–Young 1989), refraction (Bennett 1982 /
//! Sæmundsson 1986), the parallactic angle, and the analytic
//! transit/crossing solver are written fresh, validated against published
//! values in the inline and integration tests.
//!
//! All instants are folded to UTC internally; hour-angle sign follows the
//! usual convention (positive west of the meridian). Targets are precessed
//! to the epoch of the observation instant internally before any comparison
//! with sidereal time — passing J2000 catalogue coordinates directly is
//! correct (comparing J2000 RA against of-date LST without precession would
//! be off by ~2 s of RA per year, ≈13′ for J2000 targets in the mid-2020s).

use ::time::{Duration, OffsetDateTime};

use crate::angle::{parse_dec, Angle, ParseMode};
use crate::coords::{precess, Equatorial};
use crate::error::{Error, Result};
use crate::time::{julian_epoch_of, lst};

/// Degrees the hour angle advances per mean solar day (sidereal rate).
const SIDEREAL_RATE_DEG_PER_DAY: f64 = 360.985_647_366_29;

// ── Location ───────────────────────────────────────────────────────────────────

/// An observing site: latitude, east-positive longitude, and elevation.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Location {
    latitude: Angle,
    longitude: Angle,
    elevation_m: f64,
}

impl Location {
    /// Build a site from typed angles.
    ///
    /// # Errors
    /// [`Error::OutOfRange`] unless latitude ∈ `[-90°, +90°]`, longitude ∈
    /// `[-180°, +180°]`, and elevation is finite.
    pub fn new(latitude: Angle, longitude: Angle, elevation_m: f64) -> Result<Self> {
        let lat = latitude.degrees();
        if !lat.is_finite() || !(-90.0..=90.0).contains(&lat) {
            return Err(Error::OutOfRange {
                what: "latitude",
                value: lat,
            });
        }
        let lon = longitude.degrees();
        if !lon.is_finite() || !(-180.0..=180.0).contains(&lon) {
            return Err(Error::OutOfRange {
                what: "longitude",
                value: lon,
            });
        }
        if !elevation_m.is_finite() {
            return Err(Error::OutOfRange {
                what: "elevation",
                value: elevation_m,
            });
        }
        Ok(Self {
            latitude,
            longitude,
            elevation_m,
        })
    }

    /// Parse a site from strings: decimal degrees (`"52.09"`), sexagesimal
    /// FITS `SITELAT`/`SITELONG` shapes (`"+52 05 32"`, `"-111:36:00"`), or
    /// hemisphere-suffixed (`"52.09 N"`, `"4.31 W"`), where the suffix
    /// supplies the sign.
    ///
    /// # Errors
    /// [`Error::ParseCoord`] on malformed input, a suffix on the wrong axis
    /// (`N`/`S` are latitude-only, `E`/`W` longitude-only), or a suffix
    /// contradicting an explicit sign; [`Error::OutOfRange`] as in [`new`].
    ///
    /// [`new`]: Location::new
    pub fn parse(lat: &str, lon: &str, elevation_m: f64) -> Result<Self> {
        let latitude = parse_site_angle(lat, ['N', 'S'])?;
        let longitude = parse_site_angle(lon, ['E', 'W'])?;
        Self::new(latitude, longitude, elevation_m)
    }

    /// Geodetic latitude (north-positive).
    pub fn latitude(self) -> Angle {
        self.latitude
    }

    /// Longitude, east-positive.
    pub fn longitude(self) -> Angle {
        self.longitude
    }

    /// Elevation above sea level, in metres.
    pub fn elevation_m(self) -> f64 {
        self.elevation_m
    }
}

/// Parse one site angle; `hemis` is `[positive, negative]` for this axis.
fn parse_site_angle(s: &str, hemis: [char; 2]) -> Result<Angle> {
    let t = s.trim();
    let (body, hemi) = match t.chars().last() {
        Some(c) if c.is_ascii_alphabetic() => {
            let u = c.to_ascii_uppercase();
            if !"NSEW".contains(u) {
                return Err(Error::ParseCoord(t.to_string()));
            }
            if !hemis.contains(&u) {
                return Err(Error::ParseCoord(format!(
                    "hemisphere suffix {u:?} is not valid here: {t:?}"
                )));
            }
            (t[..t.len() - 1].trim_end(), Some(u))
        }
        _ => (t, None),
    };
    let angle = parse_dec(body, ParseMode::Lenient)?;
    match hemi {
        None => Ok(angle),
        Some(_) if angle.degrees().is_sign_negative() => Err(Error::ParseCoord(format!(
            "explicit sign contradicts hemisphere suffix: {t:?}"
        ))),
        Some(h) if h == hemis[1] => Ok(-angle),
        Some(_) => Ok(angle),
    }
}

// ── Horizontal coordinates ─────────────────────────────────────────────────────

/// A horizontal (alt-azimuth) position. Azimuth is measured from North
/// through East (N = 0°, E = 90°), normalized to `[0°, 360°)`.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Horizontal {
    /// Elevation above the horizon, `[-90°, +90°]`.
    pub altitude: Angle,
    /// Bearing from true North through East, `[0°, 360°)`.
    pub azimuth: Angle,
}

/// Hour angle of `target` (LST − of-date RA), normalized to `(-180°, +180°]`;
/// positive west of the meridian. The target is precessed to the epoch of
/// `at` first, so J2000 coordinates are compared against of-date sidereal
/// time correctly.
pub fn hour_angle(target: Equatorial, at: OffsetDateTime, site: &Location) -> Angle {
    let of_date = precess(target, julian_epoch_of(at));
    (lst(at, site.longitude()) - of_date.ra()).normalized_pm_180()
}

/// Horizontal position of `target` for an observer, ported from astro-math's
/// spherical-triangle formulation (geometric altitude — no refraction). The
/// target is precessed to the epoch of `at` internally.
pub fn alt_az(target: Equatorial, at: OffsetDateTime, site: &Location) -> Horizontal {
    let target = precess(target, julian_epoch_of(at));
    let ha = hour_angle(target, at, site).radians();
    let dec = target.dec().radians();
    let lat = site.latitude().radians();

    let sin_alt = dec.sin() * lat.sin() + dec.cos() * lat.cos() * ha.cos();
    let alt = sin_alt.clamp(-1.0, 1.0).asin();

    let denominator = alt.cos() * lat.cos();
    let az_deg = if denominator.abs() < 1e-10 {
        // Zenith or a polar observer: azimuth is degenerate; report the
        // meridian side implied by the hour angle (astro-math convention).
        if ha.sin() > 0.0 {
            180.0
        } else {
            0.0
        }
    } else {
        let cos_az = ((dec.sin() - alt.sin() * lat.sin()) / denominator).clamp(-1.0, 1.0);
        let az = cos_az.acos().to_degrees();
        // East of the meridian the bearing is direct; west it mirrors.
        if ha.sin() > 0.0 {
            360.0 - az
        } else {
            az
        }
    };

    Horizontal {
        altitude: Angle::from_radians(alt),
        azimuth: Angle::from_degrees(az_deg).normalized_0_360(),
    }
}

// ── Airmass & refraction ───────────────────────────────────────────────────────

/// Relative airmass for an *apparent* altitude — Kasten & Young (1989),
/// accurate to ~1% above 5° and well-behaved to the horizon. ≈1.0 at zenith,
/// ≈38 at the horizon.
///
/// # Errors
/// [`Error::OutOfRange`] below −1° (the formula's validity edge) or above
/// +90°.
pub fn airmass(altitude: Angle) -> Result<f64> {
    let h = checked_altitude(altitude)?;
    Ok(1.0 / (h.to_radians().sin() + 0.50572 * (h + 6.079_95).powf(-1.636_4)))
}

/// True (geometric) altitude from an apparent one — Bennett (1982), standard
/// conditions (1010 hPa, 10 °C), accurate to ~0.1′.
///
/// # Errors
/// [`Error::OutOfRange`] outside `[-1°, +90°]`.
pub fn refraction_apparent_to_true(apparent_alt: Angle) -> Result<Angle> {
    let h = checked_altitude(apparent_alt)?;
    let r_arcmin = 1.0 / (h + 7.31 / (h + 4.4)).to_radians().tan();
    Ok(apparent_alt - Angle::from_arcminutes(r_arcmin.max(0.0)))
}

/// Apparent altitude from a true (geometric) one — Sæmundsson (1986), the
/// standard inverse companion to Bennett, same standard conditions.
///
/// # Errors
/// [`Error::OutOfRange`] outside `[-1°, +90°]`.
pub fn refraction_true_to_apparent(true_alt: Angle) -> Result<Angle> {
    let h = checked_altitude(true_alt)?;
    let r_arcmin = 1.02 / (h + 10.3 / (h + 5.11)).to_radians().tan();
    Ok(true_alt + Angle::from_arcminutes(r_arcmin.max(0.0)))
}

/// Shared domain gate for the altitude-driven formulas, tolerant of float
/// noise a hair above the zenith.
fn checked_altitude(altitude: Angle) -> Result<f64> {
    let h = altitude.degrees();
    if !h.is_finite() || !(-1.0..=90.0 + 1e-9).contains(&h) {
        return Err(Error::OutOfRange {
            what: "altitude",
            value: h,
        });
    }
    Ok(h.min(90.0))
}

// ── Parallactic angle ──────────────────────────────────────────────────────────

/// Parallactic angle `q = atan2(sin H, tan φ cos δ − sin δ cos H)` in
/// `(-180°, +180°]`: 0 at transit (for δ < φ), negative east of the
/// meridian, positive west — the position angle of the zenith measured at
/// the target. The target is precessed to the epoch of `at` internally.
pub fn parallactic_angle(target: Equatorial, at: OffsetDateTime, site: &Location) -> Angle {
    let target = precess(target, julian_epoch_of(at));
    let ha = hour_angle(target, at, site).radians();
    let dec = target.dec().radians();
    let lat = site.latitude().radians();
    Angle::from_radians(ha.sin().atan2(lat.tan() * dec.cos() - dec.sin() * ha.cos()))
}

// ── Transit & altitude crossings ───────────────────────────────────────────────

/// The meridian transit of `target` nearest to `near` (upper culmination,
/// hour angle 0), found analytically from the sidereal rate.
pub fn transit(target: Equatorial, near: OffsetDateTime, site: &Location) -> OffsetDateTime {
    let mut t = near;
    // One step is exact up to the polynomial's tiny curvature; the second
    // absorbs it.
    for _ in 0..2 {
        let ha = hour_angle(target, t, site).degrees();
        t -= Duration::seconds_f64(ha / SIDEREAL_RATE_DEG_PER_DAY * 86_400.0);
    }
    t
}

/// Outcome of an altitude-threshold crossing query for a fixed target over
/// the day around a transit.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CrossingOutcome {
    /// The target never descends to the threshold (e.g. circumpolar above it).
    AlwaysAbove,
    /// The target never reaches the threshold.
    NeverAbove,
    /// The target crosses the threshold: rising instant, then setting
    /// instant, bracketing the transit nearest `night_of`. A graze at the
    /// threshold reports `rise == set`.
    Crosses {
        /// Upward crossing (UTC).
        rise: OffsetDateTime,
        /// Downward crossing (UTC).
        set: OffsetDateTime,
    },
}

/// When `target` rises above and sets below `threshold` altitude, around the
/// transit nearest `night_of` — the analytic solution
/// `cos H₀ = (sin h₀ − sin φ sin δ) / (cos φ cos δ)` (geometric altitude, no
/// refraction; apply [`refraction_true_to_apparent`]'s inverse to the
/// threshold first if apparent-altitude semantics are wanted).
pub fn altitude_crossings(
    target: Equatorial,
    threshold: Angle,
    night_of: OffsetDateTime,
    site: &Location,
) -> CrossingOutcome {
    let target = precess(target, julian_epoch_of(night_of));
    let phi = site.latitude().radians();
    let dec = target.dec().radians();
    let sin_h0 = threshold.radians().sin();

    let denominator = phi.cos() * dec.cos();
    if denominator.abs() < 1e-12 {
        // Site at a pole or target at a celestial pole: altitude is constant.
        return if phi.sin() * dec.sin() >= sin_h0 {
            CrossingOutcome::AlwaysAbove
        } else {
            CrossingOutcome::NeverAbove
        };
    }

    let cos_h0 = (sin_h0 - phi.sin() * dec.sin()) / denominator;
    if cos_h0 < -1.0 {
        return CrossingOutcome::AlwaysAbove;
    }
    if cos_h0 > 1.0 {
        return CrossingOutcome::NeverAbove;
    }

    let semi_arc_deg = cos_h0.acos().to_degrees();
    let half = Duration::seconds_f64(semi_arc_deg / SIDEREAL_RATE_DEG_PER_DAY * 86_400.0);
    let t0 = transit(target, night_of, site);
    CrossingOutcome::Crosses {
        rise: t0 - half,
        set: t0 + half,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::time::macros::datetime;

    fn kitt_peak() -> Location {
        Location::new(
            Angle::from_degrees(31.9583),
            Angle::from_degrees(-111.6),
            2120.0,
        )
        .unwrap()
    }

    #[test]
    fn location_validates_domains() {
        let ok = |lat: f64, lon: f64| {
            Location::new(Angle::from_degrees(lat), Angle::from_degrees(lon), 0.0)
        };
        assert!(ok(90.0, 180.0).is_ok());
        assert!(ok(-90.0, -180.0).is_ok());
        assert!(ok(90.1, 0.0).is_err());
        assert!(ok(0.0, 180.1).is_err());
        assert!(
            Location::new(Angle::from_degrees(0.0), Angle::from_degrees(0.0), f64::NAN).is_err()
        );
    }

    #[test]
    fn parse_accepts_decimal_sexagesimal_and_suffix() {
        // FITS SITELAT/SITELONG shapes.
        let l = Location::parse("+52 05 32", "-111 36 00", 0.0).unwrap();
        assert!((l.latitude().degrees() - 52.092_222).abs() < 3e-5); // ±0.1″
        assert!((l.longitude().degrees() + 111.6).abs() < 3e-5);
        // Hemisphere suffixes supply the sign.
        let l = Location::parse("52.09 S", "4.31 W", 0.0).unwrap();
        assert!((l.latitude().degrees() + 52.09).abs() < 1e-9);
        assert!((l.longitude().degrees() + 4.31).abs() < 1e-9);
        let l = Location::parse("31:57:30 N", "111:36:00 E", 2120.0).unwrap();
        assert!((l.latitude().degrees() - 31.958_333).abs() < 3e-5);
        assert!((l.longitude().degrees() - 111.6).abs() < 3e-5);
    }

    #[test]
    fn parse_rejects_wrong_axis_conflicts_and_garbage() {
        assert!(Location::parse("52.09 E", "4.31", 0.0).is_err()); // E on latitude
        assert!(Location::parse("52.09", "4.31 N", 0.0).is_err()); // N on longitude
        assert!(Location::parse("-52.09 S", "4.31", 0.0).is_err()); // sign conflict
        assert!(Location::parse("52 xx 09", "4.31", 0.0).is_err());
        assert!(Location::parse("", "4.31", 0.0).is_err());
    }

    #[test]
    fn airmass_matches_kasten_young_published_values() {
        // Kasten & Young (1989): X(90°) ≈ 1.000, X(30°) ≈ 1.994, X(0°) ≈ 37.9.
        assert!((airmass(Angle::from_degrees(90.0)).unwrap() - 1.0).abs() < 1e-3);
        assert!((airmass(Angle::from_degrees(30.0)).unwrap() - 1.994).abs() < 5e-3);
        assert!((airmass(Angle::from_degrees(0.0)).unwrap() - 37.9).abs() < 0.2);
        assert!(airmass(Angle::from_degrees(-2.0)).is_err());
    }

    #[test]
    fn refraction_matches_published_values() {
        // Bennett (1982): R ≈ 34.5′ at the apparent horizon, ≈ 1.0′ at 45°.
        let horizon = refraction_apparent_to_true(Angle::from_degrees(0.0)).unwrap();
        assert!((horizon.arcminutes() + 34.5).abs() < 0.2, "{horizon:?}");
        let mid = refraction_apparent_to_true(Angle::from_degrees(45.0)).unwrap();
        assert!(((45.0 - mid.degrees()) * 60.0 - 1.0).abs() < 0.05);
        // Sæmundsson (1986): R ≈ 28.9′ at true altitude 0.
        let apparent = refraction_true_to_apparent(Angle::from_degrees(0.0)).unwrap();
        assert!((apparent.arcminutes() - 28.9).abs() < 0.3, "{apparent:?}");
        // Zenith: no negative refraction leaks out of either direction.
        let z = Angle::from_degrees(90.0);
        assert!(refraction_apparent_to_true(z).unwrap().degrees() <= 90.0);
        assert!(refraction_true_to_apparent(z).unwrap().degrees() >= 90.0);
        assert!(refraction_apparent_to_true(Angle::from_degrees(-1.5)).is_err());
    }

    #[test]
    fn bennett_and_saemundsson_are_mutual_inverses_at_altitude() {
        for h in [5.0, 10.0, 20.0, 45.0, 70.0, 89.0] {
            let true_alt = Angle::from_degrees(h);
            let apparent = refraction_true_to_apparent(true_alt).unwrap();
            let back = refraction_apparent_to_true(apparent).unwrap();
            let drift_arcmin = (back.degrees() - h) * 60.0;
            assert!(drift_arcmin.abs() < 0.2, "h={h}: drift {drift_arcmin}′");
        }
    }

    #[test]
    fn transit_lands_at_zero_hour_angle() {
        // M31 from Kitt Peak; 5 s of time = 0.0209° of hour angle.
        let m31 =
            Equatorial::j2000(Angle::from_degrees(10.6847), Angle::from_degrees(41.2688)).unwrap();
        let site = kitt_peak();
        let t = transit(m31, datetime!(2026-07-11 09:00 UTC), &site);
        let ha = hour_angle(m31, t, &site).degrees();
        assert!(ha.abs() < 0.021, "residual HA {ha}°");
        // Nearest transit: within ±12 h (sidereal) of the seed.
        let dt = t - datetime!(2026-07-11 09:00 UTC);
        assert!(dt.whole_hours().abs() <= 12, "{dt}");
    }
}
