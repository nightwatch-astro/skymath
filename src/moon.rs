// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Lunar position, separation, illumination, rise/set, and moon-avoidance.
//!
//! Provenance: the geocentric position implements the truncated ELP-2000/82
//! theory of Meeus, *Astronomical Algorithms* 2nd ed., ch. 47; the periodic
//! term tables (47.A / 47.B) are ported from `saurvs/astro-rust`
//! (`src/lunar.rs`, MIT — see `NOTICE`).
//! Topocentric correction is Meeus ch. 40 (written fresh); illumination is
//! Meeus ch. 48; the moon-avoidance Lorentzian is the classic ACP/Berry
//! criterion. Accuracy: ~10″ (λ) / 4″ (β) geocentric truncation; treating
//! UTC as dynamical time adds ≤ ~40″ (the Moon moves ~0.55″/s of ΔT) — the
//! documented public claim is ≤2′ vs AstroPy, comfortably absorbed by the
//! planning contract for site-relative use once parallax (up to ~61′!) is
//! corrected via [`moon_position_topocentric`].

use ::time::OffsetDateTime;

use crate::angle::Angle;
use crate::coords::{precess, separation, Equatorial};
use crate::frames::mean_obliquity;
use crate::observer::{moving_body_crossings, CrossingOutcome, Location};
use crate::sun::solar_coords;
use crate::time::{julian_date, julian_epoch_of, lst};

const J2000_JD: f64 = 2_451_545.0;
/// Kilometres per astronomical unit (IAU 2012).
const KM_PER_AU: f64 = 149_597_870.7;
/// Mean synodic elongation rate of the Moon from the Sun, degrees/day.
const SYNODIC_RATE_DEG_PER_DAY: f64 = 12.190_749;

/// Periodic terms for lunar longitude (Σl, 1e-6 °) and distance (Σr, 1e-3 km):
/// multiples of (D, M, M′, F) with the two coefficients.
/// Meeus table 47.A, ported from astro-rust `lunar.rs`.
#[rustfmt::skip]
const LR_TERMS: [(i8, i8, i8, i8, i32, i32); 60] = [
    (0, 0, 1, 0, 6_288_774, -20_905_355), (2, 0, -1, 0, 1_274_027, -3_699_111),
    (2, 0, 0, 0, 658_314, -2_955_968),    (0, 0, 2, 0, 213_618, -569_925),
    (0, 1, 0, 0, -185_116, 48_888),       (0, 0, 0, 2, -114_332, -3_149),
    (2, 0, -2, 0, 58_793, 246_158),       (2, -1, -1, 0, 57_066, -152_138),
    (2, 0, 1, 0, 53_322, -170_733),       (2, -1, 0, 0, 45_758, -204_586),
    (0, 1, -1, 0, -40_923, -129_620),     (1, 0, 0, 0, -34_720, 108_743),
    (0, 1, 1, 0, -30_383, 104_755),       (2, 0, 0, -2, 15_327, 10_321),
    (0, 0, 1, 2, -12_528, 0),             (0, 0, 1, -2, 10_980, 79_661),
    (4, 0, -1, 0, 10_675, -34_782),       (0, 0, 3, 0, 10_034, -23_210),
    (4, 0, -2, 0, 8_548, -21_636),        (2, 1, -1, 0, -7_888, 24_208),
    (2, 1, 0, 0, -6_766, 30_824),         (1, 0, -1, 0, -5_163, -8_379),
    (1, 1, 0, 0, 4_987, -16_675),         (2, -1, 1, 0, 4_036, -12_831),
    (2, 0, 2, 0, 3_994, -10_445),         (4, 0, 0, 0, 3_861, -11_650),
    (2, 0, -3, 0, 3_665, 14_403),         (0, 1, -2, 0, -2_689, -7_003),
    (2, 0, -1, 2, -2_602, 0),             (2, -1, -2, 0, 2_390, 10_056),
    (1, 0, 1, 0, -2_348, 6_322),          (2, -2, 0, 0, 2_236, -9_884),
    (0, 1, 2, 0, -2_120, 5_751),          (0, 2, 0, 0, -2_069, 0),
    (2, -2, -1, 0, 2_048, -4_950),        (2, 0, 1, -2, -1_773, 4_130),
    (2, 0, 0, 2, -1_595, 0),              (4, -1, -1, 0, 1_215, -3_958),
    (0, 0, 2, 2, -1_110, 0),              (3, 0, -1, 0, -892, 3_258),
    (2, 1, 1, 0, -810, 2_616),            (4, -1, -2, 0, 759, -1_897),
    (0, 2, -1, 0, -713, -2_117),          (2, 2, -1, 0, -700, 2_354),
    (2, 1, -2, 0, 691, 0),                (2, -1, 0, -2, 596, 0),
    (4, 0, 1, 0, 549, -1_423),            (0, 0, 4, 0, 537, -1_117),
    (4, -1, 0, 0, 520, -1_571),           (1, 0, -2, 0, -487, -1_739),
    (2, 1, 0, -2, -399, 0),               (0, 0, 2, -2, -381, -4_421),
    (1, 1, 1, 0, 351, 0),                 (3, 0, -2, 0, -340, 0),
    (4, 0, -3, 0, 330, 0),                (2, -1, 2, 0, 327, 0),
    (0, 2, 1, 0, -323, 1_165),            (1, 1, -1, 0, 299, 0),
    (2, 0, 3, 0, 294, 0),                 (2, 0, -1, -2, 0, 8_752),
];

/// Periodic terms for lunar latitude (Σb, 1e-6 °). Meeus table 47.B, ported
/// from astro-rust `lunar.rs`.
#[rustfmt::skip]
const B_TERMS: [(i8, i8, i8, i8, i32); 60] = [
    (0, 0, 0, 1, 5_128_122), (0, 0, 1, 1, 280_602),  (0, 0, 1, -1, 277_693),
    (2, 0, 0, -1, 173_237),  (2, 0, -1, 1, 55_413),  (2, 0, -1, -1, 46_271),
    (2, 0, 0, 1, 32_573),    (0, 0, 2, 1, 17_198),   (2, 0, 1, -1, 9_266),
    (0, 0, 2, -1, 8_822),    (2, -1, 0, -1, 8_216),  (2, 0, -2, -1, 4_324),
    (2, 0, 1, 1, 4_200),     (2, 1, 0, -1, -3_359),  (2, -1, -1, 1, 2_463),
    (2, -1, 0, 1, 2_211),    (2, -1, -1, -1, 2_065), (0, 1, -1, -1, -1_870),
    (4, 0, -1, -1, 1_828),   (0, 1, 0, 1, -1_794),   (0, 0, 0, 3, -1_749),
    (0, 1, -1, 1, -1_565),   (1, 0, 0, 1, -1_491),   (0, 1, 1, 1, -1_475),
    (0, 1, 1, -1, -1_410),   (0, 1, 0, -1, -1_344),  (1, 0, 0, -1, -1_335),
    (0, 0, 3, 1, 1_107),     (4, 0, 0, -1, 1_021),   (4, 0, -1, 1, 833),
    (0, 0, 1, -3, 777),      (4, 0, -2, 1, 671),     (2, 0, 0, -3, 607),
    (2, 0, 2, -1, 596),      (2, -1, 1, -1, 491),    (2, 0, -2, 1, -451),
    (0, 0, 3, -1, 439),      (2, 0, 2, 1, 422),      (2, 0, -3, -1, 421),
    (2, 1, -1, 1, -366),     (2, 1, 0, 1, -351),     (4, 0, 0, 1, 331),
    (2, -1, 1, 1, 315),      (2, -2, 0, -1, 302),    (0, 0, 1, 3, -283),
    (2, 1, 1, -1, -229),     (1, 1, 0, -1, 223),     (1, 1, 0, 1, 223),
    (0, 1, -2, -1, -220),    (2, 1, -1, -1, -220),   (1, 0, 1, 1, -185),
    (2, -1, -2, -1, 181),    (0, 1, 2, 1, -177),     (4, 0, -2, -1, 176),
    (4, -1, -1, -1, 166),    (1, 0, 1, -1, -164),    (4, 0, 1, -1, 132),
    (1, 0, -1, -1, -119),    (4, -1, 0, -1, 115),    (2, -2, 0, 1, 107),
];

/// Geocentric ecliptic λ, β (degrees, mean equinox of date) and distance Δ
/// (km) at `at` — Meeus ch. 47.
pub(crate) fn lunar_coords(at: OffsetDateTime) -> (f64, f64, f64) {
    let t = (julian_date(at) - J2000_JD) / 36_525.0;

    // Fundamental arguments (Meeus 47.1–47.6), degrees.
    let l1 = 218.316_447_7 + 481_267.881_234_21 * t - 0.001_578_6 * t * t + t * t * t / 538_841.0
        - t * t * t * t / 65_194_000.0;
    let d = 297.850_192_1 + 445_267.111_403_4 * t - 0.001_881_9 * t * t + t * t * t / 545_868.0
        - t * t * t * t / 113_065_000.0;
    let m = 357.529_109_2 + 35_999.050_290_9 * t - 0.000_153_6 * t * t + t * t * t / 24_490_000.0;
    let m1 = 134.963_396_4 + 477_198.867_505_5 * t + 0.008_741_4 * t * t + t * t * t / 69_699.0
        - t * t * t * t / 14_712_000.0;
    let f = 93.272_095_0 + 483_202.017_523_3 * t - 0.003_653_9 * t * t - t * t * t / 3_526_000.0
        + t * t * t * t / 863_310_000.0;
    let e = 1.0 - 0.002_516 * t - 0.000_007_4 * t * t;

    let a1 = (119.75 + 131.849 * t).to_radians();
    let a2 = (53.09 + 479_264.29 * t).to_radians();
    let a3 = (313.45 + 481_266.484 * t).to_radians();
    let (l1_rad, d, m, m1, f) = (
        l1.to_radians(),
        d.to_radians(),
        m.to_radians(),
        m1.to_radians(),
        f.to_radians(),
    );

    let ecc = |mult: i8| match mult.abs() {
        1 => e,
        2 => e * e,
        _ => 1.0,
    };

    let (mut sum_l, mut sum_r, mut sum_b) = (0.0, 0.0, 0.0);
    for &(td, tm, tm1, tf, cl, cr) in &LR_TERMS {
        let arg = f64::from(td) * d + f64::from(tm) * m + f64::from(tm1) * m1 + f64::from(tf) * f;
        sum_l += f64::from(cl) * ecc(tm) * arg.sin();
        sum_r += f64::from(cr) * ecc(tm) * arg.cos();
    }
    for &(td, tm, tm1, tf, cb) in &B_TERMS {
        let arg = f64::from(td) * d + f64::from(tm) * m + f64::from(tm1) * m1 + f64::from(tf) * f;
        sum_b += f64::from(cb) * ecc(tm) * arg.sin();
    }

    // Additive corrections (Venus, Jupiter, flattening terms).
    sum_l += 3_958.0 * a1.sin() + 1_962.0 * (l1_rad - f).sin() + 318.0 * a2.sin();
    sum_b += -2_235.0 * l1_rad.sin()
        + 382.0 * a3.sin()
        + 175.0 * ((a1 - f).sin() + (a1 + f).sin())
        + 127.0 * (l1_rad - m1).sin()
        - 115.0 * (l1_rad + m1).sin();

    let lambda = (l1 + sum_l / 1e6).rem_euclid(360.0);
    let beta = sum_b / 1e6;
    let delta_km = 385_000.56 + sum_r / 1e3;
    (lambda, beta, delta_km)
}

/// Ecliptic-of-date (λ, β) → equatorial-of-date, via the mean obliquity.
fn ecliptic_to_equatorial(lambda_deg: f64, beta_deg: f64, at: OffsetDateTime) -> Equatorial {
    let (l, b) = (lambda_deg.to_radians(), beta_deg.to_radians());
    let eps = mean_obliquity(at);
    let ra = (l.sin() * eps.cos() - b.tan() * eps.sin()).atan2(l.cos());
    let dec = (b.sin() * eps.cos() + b.cos() * eps.sin() * l.sin())
        .clamp(-1.0, 1.0)
        .asin();
    Equatorial::at_epoch(
        Angle::from_radians(ra).normalized_0_360(),
        Angle::from_radians(dec),
        julian_epoch_of(at),
    )
    .expect("rotation output is in domain by construction")
}

/// Geocentric lunar position (equatorial, epoch of date).
///
/// Truncated ELP-2000/82 (Meeus ch. 47): ~10″ in longitude against the full
/// theory; documented public claim ≤2′ vs AstroPy (which uses a fuller
/// theory; treating UTC as dynamical time contributes ≤ ~40″). For anything
/// site-relative use [`moon_position_topocentric`] — lunar parallax reaches
/// ~61′.
///
/// ```
/// use skymath::moon_position;
/// use time::OffsetDateTime;
///
/// let geocentric = moon_position(OffsetDateTime::now_utc());
/// assert!(geocentric.dec().degrees().abs() <= 90.0);
/// ```
pub fn moon_position(at: OffsetDateTime) -> Equatorial {
    let (lambda, beta, _) = lunar_coords(at);
    ecliptic_to_equatorial(lambda, beta, at)
}

/// Moon–Earth centre distance in kilometres.
///
/// ```
/// use skymath::moon_distance_km;
/// use time::macros::datetime;
///
/// // Meeus example 47.a.
/// let km = moon_distance_km(datetime!(1992-04-12 00:00 UTC));
/// assert!((km - 368_409.7).abs() < 1.0);
/// ```
pub fn moon_distance_km(at: OffsetDateTime) -> f64 {
    lunar_coords(at).2
}

/// Topocentric lunar position for an observer (equatorial, epoch of date) —
/// Meeus ch. 40 parallax correction with WGS-84-flattened site coordinates.
/// The shift from [`moon_position`] (geocentric) can reach ~61′ (the Moon's
/// horizontal parallax).
///
/// ```
/// use skymath::{moon_position, moon_position_topocentric, separation, Location};
/// use time::OffsetDateTime;
///
/// let site = Location::parse("+52 05 32", "+004 18 27", 6.0)?;
/// let now = OffsetDateTime::now_utc();
/// let parallax_shift = separation(moon_position(now), moon_position_topocentric(now, &site));
/// assert!(parallax_shift.arcminutes() <= 62.0);
/// # Ok::<(), skymath::Error>(())
/// ```
pub fn moon_position_topocentric(at: OffsetDateTime, site: &Location) -> Equatorial {
    let (lambda, beta, delta_km) = lunar_coords(at);
    let geocentric = ecliptic_to_equatorial(lambda, beta, at);
    let (ra, dec) = (geocentric.ra().radians(), geocentric.dec().radians());

    // Observer's geocentric coordinates (Meeus ch. 11).
    let phi = site.latitude().radians();
    let u = (0.996_647_19 * phi.tan()).atan();
    let h_frac = site.elevation_m() / 6_378_140.0;
    let rho_sin = 0.996_647_19 * u.sin() + h_frac * phi.sin();
    let rho_cos = u.cos() + h_frac * phi.cos();

    // Equatorial horizontal parallax and local hour angle.
    let sin_pi = 6_378.14 / delta_km;
    let h = (lst(at, site.longitude()) - geocentric.ra())
        .normalized_pm_180()
        .radians();

    // Meeus 40.2 / 40.3.
    let delta_ra = (-rho_cos * sin_pi * h.sin()).atan2(dec.cos() - rho_cos * sin_pi * h.cos());
    let dec_top = ((dec.sin() - rho_sin * sin_pi) * delta_ra.cos())
        .atan2(dec.cos() - rho_cos * sin_pi * h.cos());

    Equatorial::at_epoch(
        Angle::from_radians(ra + delta_ra).normalized_0_360(),
        Angle::from_radians(dec_top),
        julian_epoch_of(at),
    )
    .expect("parallax correction stays in domain")
}

/// Great-circle separation between the topocentric Moon and `target` at an
/// instant. The target is precessed to the epoch of date internally, matching
/// the observer-module semantics.
///
/// ```
/// use skymath::{lunar_separation, Equatorial, Location, ParseMode};
/// use time::OffsetDateTime;
///
/// let m31 = Equatorial::parse_j2000("00:42:44.3", "+41:16:09", ParseMode::Strict)?;
/// let site = Location::parse("+52 05 32", "+004 18 27", 6.0)?;
/// let sep = lunar_separation(m31, OffsetDateTime::now_utc(), &site);
/// assert!((0.0..=180.0).contains(&sep.degrees()));
/// # Ok::<(), skymath::Error>(())
/// ```
pub fn lunar_separation(target: Equatorial, at: OffsetDateTime, site: &Location) -> Angle {
    let target = precess(target, julian_epoch_of(at));
    separation(moon_position_topocentric(at, site), target)
}

/// When the topocentric Moon rises above and sets below `threshold` altitude,
/// around the lunar transit nearest `night_of` (moving-body iteration of the
/// analytic solver; the Moon moves ~13°/day). Matches astroplan within ±3 min.
///
/// ```
/// use skymath::{moon_crossings, Angle, CrossingOutcome, Location};
/// use time::OffsetDateTime;
///
/// let site = Location::parse("+52 05 32", "+004 18 27", 6.0)?;
/// match moon_crossings(Angle::from_degrees(0.0), OffsetDateTime::now_utc(), &site) {
///     CrossingOutcome::Crosses { rise, set } => println!("Moon: {rise} -> {set}"),
///     outcome => println!("{outcome:?}"),
/// }
/// # Ok::<(), skymath::Error>(())
/// ```
pub fn moon_crossings(
    threshold: Angle,
    night_of: OffsetDateTime,
    site: &Location,
) -> CrossingOutcome {
    moving_body_crossings(
        |t| moon_position_topocentric(t, site),
        threshold,
        night_of,
        site,
        0.0,
    )
}

/// The Moon's phase angle `i` (Sun–Moon–Earth angle, Meeus ch. 48 exact
/// form): 0° at full Moon, 180° at new. Feeds [`moon_illumination`] and
/// [`moon_avoidance_lorentzian`].
///
/// ```
/// use skymath::moon_phase_angle;
/// use time::macros::datetime;
///
/// // Meeus example 48.a.
/// let i = moon_phase_angle(datetime!(1992-04-12 00:00 UTC)).degrees();
/// assert!((i - 69.0756).abs() < 0.05);
/// ```
pub fn moon_phase_angle(at: OffsetDateTime) -> Angle {
    let (_, sun_r_au, _) = solar_coords(at);
    let r = sun_r_au * KM_PER_AU;
    let (lambda, beta, delta) = lunar_coords(at);
    // Geocentric elongation between the apparent Sun and Moon.
    let psi = separation(
        crate::sun::sun_position(at),
        ecliptic_to_equatorial(lambda, beta, at),
    )
    .radians();
    Angle::from_radians((r * psi.sin()).atan2(delta - r * psi.cos()))
}

/// Illuminated fraction of the Moon's disk, `k = (1 + cos i) / 2` ∈ [0, 1],
/// derived from [`moon_phase_angle`].
///
/// ```
/// use skymath::moon_illumination;
/// use time::macros::datetime;
///
/// // Meeus example 48.a.
/// let k = moon_illumination(datetime!(1992-04-12 00:00 UTC));
/// assert!((k - 0.6786).abs() < 0.005);
/// ```
pub fn moon_illumination(at: OffsetDateTime) -> f64 {
    (1.0 + moon_phase_angle(at).radians().cos()) / 2.0
}

/// The classic moon-avoidance Lorentzian (ACP/Berry): the required minimum
/// target–Moon separation at `at`, largest at full Moon and relaxing with
/// lunar age:
///
/// `S(d) = separation_at_full / (1 + (d / half_width_days)²)`
///
/// where `d` is the time from full Moon in days, derived from the phase angle
/// via the mean synodic rate. At full Moon it returns `separation_at_full`;
/// `half_width_days` from full it returns half of it.
///
/// ```
/// use skymath::{moon_avoidance_lorentzian, Angle};
/// use time::OffsetDateTime;
///
/// let min_separation =
///     moon_avoidance_lorentzian(Angle::from_degrees(60.0), 7.0, OffsetDateTime::now_utc());
/// assert!((0.0..=60.0).contains(&min_separation.degrees()));
/// ```
pub fn moon_avoidance_lorentzian(
    separation_at_full: Angle,
    half_width_days: f64,
    at: OffsetDateTime,
) -> Angle {
    let days_from_full = moon_phase_angle(at).degrees().abs() / SYNODIC_RATE_DEG_PER_DAY;
    let h = half_width_days.max(f64::MIN_POSITIVE);
    separation_at_full / (1.0 + (days_from_full / h).powi(2))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::time::macros::datetime;

    #[test]
    fn meeus_47a_lunar_position() {
        // Meeus example 47.a — 1992-04-12T00:00 TD (same numeric JD fed as
        // UTC, so the comparison pins the series itself, not ΔT):
        // λ = 133.162655°, β = −3.229126°, Δ = 368409.7 km.
        let (lambda, beta, delta) = lunar_coords(datetime!(1992-04-12 00:00 UTC));
        assert!((lambda - 133.162_655).abs() * 3600.0 < 1.0, "λ = {lambda}");
        assert!((beta + 3.229_126).abs() * 3600.0 < 1.0, "β = {beta}");
        assert!((delta - 368_409.7).abs() < 1.0, "Δ = {delta}");
        // Horizontal parallax π = 0.991990°.
        let pi = (6_378.14 / delta).asin().to_degrees();
        assert!((pi - 0.991_990).abs() < 1e-5, "π = {pi}");
    }

    #[test]
    fn topocentric_parallax_has_the_expected_magnitude() {
        // For a mid-latitude site with the Moon well above the horizon the
        // topocentric shift is a large fraction of the ~57′ mean horizontal
        // parallax — and never exceeds it.
        let site =
            Location::new(Angle::from_degrees(52.155), Angle::from_degrees(4.485), 6.0).unwrap();
        let at = datetime!(2026-07-11 22:00 UTC);
        let shift = separation(moon_position(at), moon_position_topocentric(at, &site));
        assert!(
            (10.0..62.0).contains(&shift.arcminutes()),
            "parallax shift {}′",
            shift.arcminutes()
        );
    }

    #[test]
    fn meeus_48a_illumination() {
        // Meeus example 48.a — 1992-04-12T00:00 TD: i = 69.0756°, k = 0.6786.
        let at = datetime!(1992-04-12 00:00 UTC);
        let i = moon_phase_angle(at).degrees();
        assert!((i - 69.075_6).abs() < 0.05, "i = {i}");
        let k = moon_illumination(at);
        assert!((k - 0.678_6).abs() < 0.005, "k = {k}");
    }

    #[test]
    fn avoidance_lorentzian_shape() {
        let at = datetime!(2026-07-11 22:00 UTC);
        let s = Angle::from_degrees(60.0);
        let d = moon_phase_angle(at).degrees().abs() / SYNODIC_RATE_DEG_PER_DAY;
        // Evaluated at the current instant the formula must equal the closed
        // form exactly.
        let expect = 60.0 / (1.0 + (d / 7.0_f64).powi(2));
        assert!((moon_avoidance_lorentzian(s, 7.0, at).degrees() - expect).abs() < 1e-9);
        // Degenerate half-width never divides by zero.
        assert!(moon_avoidance_lorentzian(s, 0.0, at).degrees() >= 0.0);
    }
}
