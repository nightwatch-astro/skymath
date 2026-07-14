// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Solar position and twilight times.
//!
//! Provenance: written fresh from Meeus, *Astronomical Algorithms* 2nd ed.,
//! ch. 25 ("low accuracy" apparent Sun — mean longitude, three-term equation
//! of centre, aberration and Ω-nutation corrections), accurate to ~0.01°
//! (36″), inside the crate's 1′ contract. Twilight solves the Sun's altitude
//! crossings with the moving-body iteration of the analytic solver. UTC is
//! used where Meeus specifies dynamical time — ΔT ≈ 70 s shifts the solar
//! longitude by ~3″, i.e. twilight instants by ~3 s.

use ::time::OffsetDateTime;

use crate::angle::Angle;
use crate::coords::Equatorial;
use crate::frames::mean_obliquity;
use crate::observer::{moving_body_crossings, CrossingOutcome, Location};
use crate::time::{julian_date, julian_epoch_of};

/// J2000.0 as a Julian date (shared with `time`; redeclared to keep the
/// module self-contained).
const J2000_JD: f64 = 2_451_545.0;

/// Apparent solar ecliptic longitude (degrees) and Sun–Earth distance (AU)
/// at `at`, plus the Ω-corrected obliquity (radians) for the same instant.
pub(crate) fn solar_coords(at: OffsetDateTime) -> (f64, f64, f64) {
    let t = (julian_date(at) - J2000_JD) / 36_525.0;

    // Mean longitude, mean anomaly, eccentricity (Meeus 25.2–25.4).
    let l0 = 280.466_46 + 36_000.769_83 * t + 0.000_303_2 * t * t;
    let m = (357.529_11 + 35_999.050_29 * t - 0.000_153_7 * t * t).to_radians();
    let e = 0.016_708_634 - 0.000_042_037 * t - 0.000_000_126_7 * t * t;

    // Equation of centre and true anomaly.
    let c = (1.914_602 - 0.004_817 * t - 0.000_014 * t * t) * m.sin()
        + (0.019_993 - 0.000_101 * t) * (2.0 * m).sin()
        + 0.000_289 * (3.0 * m).sin();
    let true_longitude = l0 + c;
    let nu = m + c.to_radians();

    // Radius vector (25.5) and apparent longitude (aberration + Ω term).
    let r = 1.000_001_018 * (1.0 - e * e) / (1.0 + e * nu.cos());
    let omega = (125.04 - 1_934.136 * t).to_radians();
    let lambda = (true_longitude - 0.005_69 - 0.004_78 * omega.sin()).rem_euclid(360.0);

    // Obliquity corrected for the same Ω term (Meeus 25.8).
    let eps = mean_obliquity(at) + (0.002_56 * omega.cos()).to_radians();
    (lambda, r, eps)
}

/// Apparent geocentric solar position (equatorial, epoch of date). Feeds
/// [`twilight`] and [`crate::moon_phase_angle`] internally.
///
/// Meeus ch. 25 low-accuracy method: ~0.01° (36″), inside the crate's ≤1′
/// contract (pinned against AstroPy `get_sun`).
///
/// ```
/// use skymath::sun_position;
/// use time::macros::datetime;
///
/// // Meeus example 25.a.
/// let sun = sun_position(datetime!(1992-10-13 00:00 UTC));
/// assert!((sun.ra().degrees() - 198.38083).abs() < 5e-3);
/// assert!((sun.dec().degrees() + 7.78507).abs() < 5e-3);
/// ```
pub fn sun_position(at: OffsetDateTime) -> Equatorial {
    let (lambda, _, eps) = solar_coords(at);
    let lambda = lambda.to_radians();

    let ra = (lambda.sin() * eps.cos()).atan2(lambda.cos());
    let dec = (eps.sin() * lambda.sin()).asin();
    Equatorial::at_epoch(
        Angle::from_radians(ra).normalized_0_360(),
        Angle::from_radians(dec),
        julian_epoch_of(at),
    )
    .expect("rotation output is in domain by construction")
}

/// A twilight definition: how far below the geometric horizon the Sun must
/// sit for the sky to count as dark.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Twilight {
    /// Sun below −6°.
    Civil,
    /// Sun below −12°.
    Nautical,
    /// Sun below −18° — fully dark for deep-sky imaging.
    Astronomical,
}

impl Twilight {
    /// The solar altitude threshold this definition uses.
    ///
    /// ```
    /// use skymath::Twilight;
    ///
    /// assert_eq!(Twilight::Astronomical.threshold().degrees(), -18.0);
    /// ```
    #[must_use]
    pub fn threshold(self) -> Angle {
        Angle::from_degrees(match self {
            Twilight::Civil => -6.0,
            Twilight::Nautical => -12.0,
            Twilight::Astronomical => -18.0,
        })
    }
}

/// Outcome of a twilight query for one night.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TwilightOutcome {
    /// Darkness begins at `dusk` and ends at `dawn` (UTC), bracketing the
    /// Sun's lower culmination nearest the queried instant.
    Night {
        /// Evening: the Sun descends through the threshold.
        dusk: OffsetDateTime,
        /// Morning: the Sun ascends back through it.
        dawn: OffsetDateTime,
    },
    /// The Sun never descends below the threshold that night (bright summer
    /// nights, midnight sun).
    NeverDark,
    /// The Sun never ascends above the threshold (polar winter): dark
    /// throughout.
    AlwaysDark,
}

/// Dusk and dawn instants for the night whose solar lower culmination is
/// nearest `night_of`, at the darkness level `kind`.
///
/// Matches astroplan's twilight instants within ±60 s (the Sun's altitude
/// changes ~1° per 4 minutes at mid-latitudes; the 0.01° solar-position
/// accuracy contributes only a few seconds).
///
/// ```
/// use skymath::{twilight, Location, Twilight, TwilightOutcome};
/// use time::OffsetDateTime;
///
/// let site = Location::parse("+52 05 32", "+004 18 27", 6.0)?;
/// match twilight(Twilight::Astronomical, OffsetDateTime::now_utc(), &site) {
///     TwilightOutcome::Night { dusk, dawn } => println!("dark {dusk} to {dawn}"),
///     TwilightOutcome::NeverDark => println!("never astronomically dark tonight"),
///     TwilightOutcome::AlwaysDark => println!("dark around the clock"),
/// }
/// # Ok::<(), skymath::Error>(())
/// ```
pub fn twilight(kind: Twilight, night_of: OffsetDateTime, site: &Location) -> TwilightOutcome {
    match moving_body_crossings(sun_position, kind.threshold(), night_of, site, 180.0) {
        CrossingOutcome::AlwaysAbove => TwilightOutcome::NeverDark,
        CrossingOutcome::NeverAbove => TwilightOutcome::AlwaysDark,
        CrossingOutcome::Crosses { rise, set } => TwilightOutcome::Night {
            dusk: set,
            dawn: rise,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::time::macros::datetime;

    #[test]
    fn meeus_25a_solar_position() {
        // Meeus example 25.a — 1992-10-13T00:00 TD (fed as UTC per R18):
        // apparent λ = 199.90895°(±ΔT drift ~0.003″), R = 0.99766 AU,
        // apparent α = 198.38083°, δ = −7.78507°.
        let at = datetime!(1992-10-13 00:00 UTC);
        let (lambda, r, _) = solar_coords(at);
        assert!((lambda - 199.908_95).abs() < 5e-4, "λ = {lambda}");
        assert!((r - 0.997_66).abs() < 1e-5, "R = {r}");

        let p = sun_position(at);
        assert!(
            (p.ra().degrees() - 198.380_83).abs() < 5e-3,
            "α {}",
            p.ra().degrees()
        );
        assert!(
            (p.dec().degrees() + 7.785_07).abs() < 5e-3,
            "δ {}",
            p.dec().degrees()
        );
    }

    #[test]
    fn twilight_thresholds() {
        for (kind, expect) in [
            (Twilight::Civil, -6.0),
            (Twilight::Nautical, -12.0),
            (Twilight::Astronomical, -18.0),
        ] {
            assert!((kind.threshold().degrees() - expect).abs() < 1e-12);
        }
    }
}
