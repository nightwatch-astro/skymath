//! Sky frame conversions: equatorial ↔ galactic and equatorial ↔ ecliptic.
//!
//! Provenance: written fresh (astro-math 0.2.1 ships no frames module).
//! Galactic uses the fixed J2000 IAU rotation (pole α = 192.85948°,
//! δ = +27.12825°, node l = 122.93192°); ecliptic uses the mean obliquity
//! ε(T), IAU-1976 polynomial (Meeus 22.2). Both are exact rotations —
//! validated against Meeus example 13.a and the galactic centre/pole anchors.

use ::time::OffsetDateTime;

use crate::angle::Angle;
use crate::coords::{precess, Epoch, Equatorial};
use crate::time::julian_epoch_of;

/// Right ascension of the north galactic pole, J2000 (degrees).
const NGP_RA_DEG: f64 = 192.859_48;
/// Declination of the north galactic pole, J2000 (degrees).
const NGP_DEC_DEG: f64 = 27.128_25;
/// Galactic longitude of the north celestial pole, J2000 (degrees).
const NCP_L_DEG: f64 = 122.931_92;

/// A galactic-frame position (IAU J2000 definition).
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Galactic {
    /// Galactic longitude `l`, `[0°, 360°)`.
    pub l: Angle,
    /// Galactic latitude `b`, `[-90°, +90°]`.
    pub b: Angle,
}

/// An ecliptic-frame position (mean ecliptic and equinox of date).
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ecliptic {
    /// Ecliptic longitude `λ`, `[0°, 360°)`.
    pub lambda: Angle,
    /// Ecliptic latitude `β`, `[-90°, +90°]`.
    pub beta: Angle,
}

// ── Galactic ───────────────────────────────────────────────────────────────────

/// Galactic coordinates of an equatorial position. Positions at another
/// epoch are precessed to J2000 first (the rotation constants are J2000).
pub fn to_galactic(eq: Equatorial) -> Galactic {
    let eq = precess(eq, Epoch::J2000);
    let (a, d) = (eq.ra().radians(), eq.dec().radians());
    let (a_g, d_g) = (NGP_RA_DEG.to_radians(), NGP_DEC_DEG.to_radians());

    let da = a - a_g;
    let sin_b = d_g.sin() * d.sin() + d_g.cos() * d.cos() * da.cos();
    let b = sin_b.clamp(-1.0, 1.0).asin();
    let l = NCP_L_DEG.to_radians()
        - (d.cos() * da.sin()).atan2(d.sin() * d_g.cos() - d.cos() * d_g.sin() * da.cos());

    Galactic {
        l: Angle::from_radians(l).normalized_0_360(),
        b: Angle::from_radians(b),
    }
}

/// Equatorial (J2000) position of galactic coordinates.
pub fn from_galactic(g: Galactic) -> Equatorial {
    let (l, b) = (g.l.radians(), g.b.radians());
    let (a_g, d_g) = (NGP_RA_DEG.to_radians(), NGP_DEC_DEG.to_radians());

    let dl = NCP_L_DEG.to_radians() - l;
    let sin_d = d_g.sin() * b.sin() + d_g.cos() * b.cos() * dl.cos();
    let dec = sin_d.clamp(-1.0, 1.0).asin();
    let ra = a_g + (b.cos() * dl.sin()).atan2(b.sin() * d_g.cos() - b.cos() * d_g.sin() * dl.cos());

    Equatorial::j2000(
        Angle::from_radians(ra).normalized_0_360(),
        Angle::from_radians(dec),
    )
    .expect("rotation output is in domain by construction")
}

// ── Ecliptic ───────────────────────────────────────────────────────────────────

/// Mean obliquity of the ecliptic ε(T), IAU-1976 polynomial (Meeus 22.2),
/// for the instant `at`.
fn mean_obliquity(at: OffsetDateTime) -> f64 {
    let t = match julian_epoch_of(at) {
        Epoch::OfDate(year) => (year - 2_000.0) / 100.0,
        Epoch::J2000 => 0.0,
    };
    let arcsec = 21.448 - 46.815_0 * t - 0.000_59 * t * t + 0.001_813 * t * t * t;
    (23.0 + 26.0 / 60.0 + arcsec / 3_600.0).to_radians()
}

/// Ecliptic coordinates (mean ecliptic of date) of an equatorial position at
/// instant `at`. The position is precessed to the epoch of date first.
pub fn to_ecliptic(eq: Equatorial, at: OffsetDateTime) -> Ecliptic {
    let eq = precess(eq, julian_epoch_of(at));
    let (a, d) = (eq.ra().radians(), eq.dec().radians());
    let eps = mean_obliquity(at);

    let lambda = (a.sin() * eps.cos() + d.tan() * eps.sin()).atan2(a.cos());
    let sin_beta = d.sin() * eps.cos() - d.cos() * eps.sin() * a.sin();
    let beta = sin_beta.clamp(-1.0, 1.0).asin();

    Ecliptic {
        lambda: Angle::from_radians(lambda).normalized_0_360(),
        beta: Angle::from_radians(beta),
    }
}

/// Equatorial (J2000) position of mean-ecliptic-of-date coordinates at
/// instant `at`.
pub fn from_ecliptic(e: Ecliptic, at: OffsetDateTime) -> Equatorial {
    let (l, b) = (e.lambda.radians(), e.beta.radians());
    let eps = mean_obliquity(at);

    let ra = (l.sin() * eps.cos() - b.tan() * eps.sin()).atan2(l.cos());
    let sin_dec = b.sin() * eps.cos() + b.cos() * eps.sin() * l.sin();
    let dec = sin_dec.clamp(-1.0, 1.0).asin();

    let of_date = Equatorial::at_epoch(
        Angle::from_radians(ra).normalized_0_360(),
        Angle::from_radians(dec),
        julian_epoch_of(at),
    )
    .expect("rotation output is in domain by construction");
    precess(of_date, Epoch::J2000)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::time::macros::datetime;

    #[test]
    fn mean_obliquity_at_j2000_is_23_4393() {
        let eps = mean_obliquity(datetime!(2000-01-01 12:00 UTC)).to_degrees();
        assert!((eps - 23.439_291_1).abs() < 1e-6, "ε = {eps}°");
    }
}
