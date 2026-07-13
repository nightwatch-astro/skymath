//! Equatorial coordinates, spherical geometry, and precession.
//!
//! Provenance: [`Epoch`], [`Equatorial`], [`separation`], and [`precess`] are
//! extracted from the sibling crate `target-match`; [`position_angle`] and the
//! tangent-offset decomposition are hoisted from its matcher and made public;
//! [`apply_offset`] (the inverse, spherical destination-point) is new here.
//!
//! Precession is IAU 1976 (Meeus ch. 21), planning grade: ≤ ~1 arcsecond over
//! several centuries. Apparent-place terms (nutation, aberration, proper
//! motion) are out of scope by design.

use crate::angle::{format_dec, format_ra, parse_dec, parse_ra, Angle, ParseMode, SexaStyle};
use crate::error::{Error, Result};

const RAD_PER_DEG: f64 = core::f64::consts::PI / 180.0;

// ── Epoch ──────────────────────────────────────────────────────────────────────

/// The reference epoch of a sky position.
///
/// `OfDate` carries a Julian year (e.g. `2026.5`) — the observation instant to
/// day precision, which is far finer than precession needs. Because the year is
/// always present, a "JNow without a date" state is unrepresentable and
/// precession is always well-defined.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Epoch {
    /// The J2000.0 standard epoch (≈ ICRS for planning-grade work).
    J2000,
    /// Epoch of date, as a Julian year.
    OfDate(f64),
}

impl Epoch {
    /// Julian centuries of this epoch measured from J2000 (`J2000` → 0).
    #[must_use]
    pub fn julian_centuries_from_j2000(self) -> f64 {
        match self {
            Epoch::J2000 => 0.0,
            Epoch::OfDate(year) => (year - 2000.0) / 100.0,
        }
    }
}

// ── Equatorial ─────────────────────────────────────────────────────────────────

/// An equatorial sky position (right ascension, declination) tagged with an
/// [`Epoch`]. After construction, `ra ∈ [0, 360)`° and `dec ∈ [-90, 90]`°.
///
/// ```
/// use skymath::{Equatorial, ParseMode, SexaStyle};
///
/// let m31 = Equatorial::parse_j2000("00:42:44.3", "+41:16:09", ParseMode::Strict)?;
/// assert_eq!(m31.ra_sexagesimal(SexaStyle::default()), "00:42:44.30");
/// assert_eq!(m31.dec_sexagesimal(SexaStyle::default()), "+41:16:09.00");
/// # Ok::<(), skymath::Error>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Equatorial {
    ra: Angle,
    dec: Angle,
    epoch: Epoch,
}

impl Equatorial {
    /// Construct from RA/Dec angles at an epoch, validating domains.
    ///
    /// # Errors
    /// [`Error::OutOfRange`] if RA ∉ [0, 360), Dec ∉ [-90, 90], or the epoch
    /// year is non-finite.
    pub fn at_epoch(ra: Angle, dec: Angle, epoch: Epoch) -> Result<Self> {
        let ra_deg = ra.degrees();
        let dec_deg = dec.degrees();
        if !ra_deg.is_finite() || !(0.0..360.0).contains(&ra_deg) {
            return Err(Error::OutOfRange {
                what: "right ascension",
                value: ra_deg,
            });
        }
        if !dec_deg.is_finite() || !(-90.0..=90.0).contains(&dec_deg) {
            return Err(Error::OutOfRange {
                what: "declination",
                value: dec_deg,
            });
        }
        if let Epoch::OfDate(year) = epoch {
            if !year.is_finite() {
                return Err(Error::OutOfRange {
                    what: "epoch year",
                    value: year,
                });
            }
        }
        Ok(Self { ra, dec, epoch })
    }

    /// Construct a J2000 position from RA/Dec angles.
    ///
    /// # Errors
    /// See [`Equatorial::at_epoch`].
    pub fn j2000(ra: Angle, dec: Angle) -> Result<Self> {
        Self::at_epoch(ra, dec, Epoch::J2000)
    }

    /// Parse RA and Dec strings (sexagesimal, or decimal in lenient mode) at an
    /// epoch. Sexagesimal RA is hours (×15); Dec is degrees.
    ///
    /// # Errors
    /// [`Error::ParseCoord`] on malformed input; [`Error::OutOfRange`] on domain.
    pub fn parse_at_epoch(ra: &str, dec: &str, epoch: Epoch, mode: ParseMode) -> Result<Self> {
        Self::at_epoch(parse_ra(ra, mode)?, parse_dec(dec, mode)?, epoch)
    }

    /// Parse a J2000 position from RA/Dec strings.
    ///
    /// # Errors
    /// See [`Equatorial::parse_at_epoch`].
    pub fn parse_j2000(ra: &str, dec: &str, mode: ParseMode) -> Result<Self> {
        Self::parse_at_epoch(ra, dec, Epoch::J2000, mode)
    }

    /// Right ascension.
    #[must_use]
    pub fn ra(self) -> Angle {
        self.ra
    }
    /// Declination.
    #[must_use]
    pub fn dec(self) -> Angle {
        self.dec
    }
    /// Reference epoch.
    #[must_use]
    pub fn epoch(self) -> Epoch {
        self.epoch
    }
    /// `(ra_degrees, dec_degrees)`.
    #[must_use]
    pub fn to_degrees(self) -> (f64, f64) {
        (self.ra.degrees(), self.dec.degrees())
    }

    /// Format RA as sexagesimal hours in the given style.
    #[must_use]
    pub fn ra_sexagesimal(self, style: SexaStyle) -> String {
        format_ra(self.ra, style)
    }
    /// Format Dec as signed sexagesimal degrees in the given style.
    #[must_use]
    pub fn dec_sexagesimal(self, style: SexaStyle) -> String {
        format_dec(self.dec, style)
    }

    /// Unit direction vector `(x, y, z)` on the celestial sphere.
    pub(crate) fn to_unit_vector(self) -> [f64; 3] {
        let (a, d) = (self.ra.radians(), self.dec.radians());
        [d.cos() * a.cos(), d.cos() * a.sin(), d.sin()]
    }

    /// Build a position from a unit vector at the given epoch (RA normalized to
    /// `[0, 360)`).
    pub(crate) fn from_unit_vector(v: [f64; 3], epoch: Epoch) -> Self {
        let ra = Angle::from_radians(v[1].atan2(v[0])).normalized_0_360();
        let dec = Angle::from_radians(v[2].atan2((v[0] * v[0] + v[1] * v[1]).sqrt()));
        Self { ra, dec, epoch }
    }
}

// ── Separation & position angle ────────────────────────────────────────────────

/// Great-circle angular separation between two positions (haversine form).
///
/// The result is in `[0, 180]`°, symmetric in its arguments, and numerically
/// stable for small separations. Epochs are not reconciled here — this is a
/// purely geometric operation on the given numbers.
///
/// ```
/// use skymath::{separation, Equatorial, ParseMode};
///
/// let m31 = Equatorial::parse_j2000("00:42:44.3", "+41:16:09", ParseMode::Strict)?;
/// let m110 = Equatorial::parse_j2000("00:40:22.1", "+41:41:07", ParseMode::Lenient)?;
/// assert!((separation(m31, m110).arcminutes() - 36.5).abs() < 1.0);
/// # Ok::<(), skymath::Error>(())
/// ```
#[must_use]
pub fn separation(a: Equatorial, b: Equatorial) -> Angle {
    let (ra1, dec1) = (a.ra.radians(), a.dec.radians());
    let (ra2, dec2) = (b.ra.radians(), b.dec.radians());
    let (dra, ddec) = (ra2 - ra1, dec2 - dec1);
    let sin_ddec = (ddec / 2.0).sin();
    let sin_dra = (dra / 2.0).sin();
    // hav(θ) = sin²(Δδ/2) + cos δ1 · cos δ2 · sin²(Δα/2)
    let h = sin_ddec.mul_add(sin_ddec, dec1.cos() * dec2.cos() * sin_dra * sin_dra);
    let central = 2.0 * h.sqrt().clamp(0.0, 1.0).asin();
    Angle::from_radians(central)
}

/// Position angle from `from` to `to`, measured East of North, in `[0, 360)`°.
///
/// At the celestial poles every direction is "south"/"north"; the atan2
/// convention there yields a defined (if arbitrary) angle rather than NaN.
///
/// ```
/// use skymath::{position_angle, Equatorial, ParseMode};
///
/// let m31 = Equatorial::parse_j2000("00:42:44.3", "+41:16:09", ParseMode::Strict)?;
/// let m110 = Equatorial::parse_j2000("00:40:22.1", "+41:41:07", ParseMode::Lenient)?;
/// // M110 sits west and slightly north of M31.
/// assert!((180.0..360.0).contains(&position_angle(m31, m110).degrees()));
/// # Ok::<(), skymath::Error>(())
/// ```
#[must_use]
pub fn position_angle(from: Equatorial, to: Equatorial) -> Angle {
    let (a0, d0) = (from.ra.radians(), from.dec.radians());
    let (a, d) = (to.ra.radians(), to.dec.radians());
    let da = a - a0;
    let y = d.cos() * da.sin();
    let x = d0.cos() * d.sin() - d0.sin() * d.cos() * da.cos();
    Angle::from_radians(y.atan2(x)).normalized_0_360()
}

// ── Tangent-plane offsets ──────────────────────────────────────────────────────

/// A sky-tangent offset decomposed into East and North components.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TangentOffset {
    /// Offset toward increasing RA (East), as an angle on the sky.
    pub east: Angle,
    /// Offset toward the North celestial pole, as an angle on the sky.
    pub north: Angle,
}

/// Decompose the great-circle arc from `from` to `to` into East/North
/// components via separation and position angle (robust polar form — never
/// divides by the cosine of the separation). Inverse of [`apply_offset`].
///
/// ```
/// use skymath::{apply_offset, separation, tangent_offset, Equatorial, ParseMode};
///
/// let m31 = Equatorial::parse_j2000("00:42:44.3", "+41:16:09", ParseMode::Strict)?;
/// let m110 = Equatorial::parse_j2000("00:40:22.1", "+41:41:07", ParseMode::Lenient)?;
/// let offset = tangent_offset(m31, m110);
/// let back = apply_offset(m31, offset);
/// assert!(separation(m110, back).arcseconds() < 1e-3);
/// # Ok::<(), skymath::Error>(())
/// ```
#[must_use]
pub fn tangent_offset(from: Equatorial, to: Equatorial) -> TangentOffset {
    let sep = separation(from, to).radians();
    let pa = position_angle(from, to).radians();
    TangentOffset {
        east: Angle::from_radians(sep * pa.sin()),
        north: Angle::from_radians(sep * pa.cos()),
    }
}

/// Apply an East/North offset to a position: travel the great circle whose
/// initial bearing is the offset's position angle for the offset's arc length
/// (spherical destination-point formula). Inverse of [`tangent_offset`].
///
/// The result keeps `from`'s epoch. Declination is clamped to the valid domain
/// against floating-point drift at the poles.
#[must_use]
pub fn apply_offset(from: Equatorial, offset: TangentOffset) -> Equatorial {
    let (e, n) = (offset.east.radians(), offset.north.radians());
    let sep = e.hypot(n);
    if sep == 0.0 {
        return from;
    }
    let pa = e.atan2(n);
    let (phi1, lam1) = (from.dec.radians(), from.ra.radians());
    let (sin_phi2_raw, cos_sep) = (
        phi1.sin() * sep.cos() + phi1.cos() * sep.sin() * pa.cos(),
        sep.cos(),
    );
    let sin_phi2 = sin_phi2_raw.clamp(-1.0, 1.0);
    let phi2 = sin_phi2.asin();
    let lam2 = lam1 + (pa.sin() * sep.sin() * phi1.cos()).atan2(cos_sep - phi1.sin() * sin_phi2);
    Equatorial {
        ra: Angle::from_radians(lam2).normalized_0_360(),
        dec: Angle::from_degrees(phi2.to_degrees().clamp(-90.0, 90.0)),
        epoch: from.epoch,
    }
}

// ── Precession (IAU 1976, Meeus ch. 21) ────────────────────────────────────────

/// Precess a position to another epoch using IAU 1976 precession.
///
/// Handles J2000 → epoch-of-date, epoch-of-date → J2000, and date → date (via
/// J2000). Accurate to ≤ ~1 arcsecond over several centuries — well inside
/// planning grade. Precessing to the same epoch is the identity.
///
/// ```
/// use skymath::{julian_epoch_of, precess, Epoch, Equatorial, ParseMode};
/// use time::OffsetDateTime;
///
/// let m31 = Equatorial::parse_j2000("00:42:44.3", "+41:16:09", ParseMode::Strict)?;
/// let tonight = julian_epoch_of(OffsetDateTime::now_utc());
/// let of_date = precess(m31, tonight);
/// assert_eq!(of_date.epoch(), tonight);
/// assert_eq!(precess(m31, Epoch::J2000), m31);
/// # Ok::<(), skymath::Error>(())
/// ```
#[must_use]
pub fn precess(pos: Equatorial, to: Epoch) -> Equatorial {
    if pos.epoch == to {
        return pos;
    }
    // Reduce to J2000 first, then forward to the target.
    let at_j2000 = match pos.epoch {
        Epoch::J2000 => pos,
        Epoch::OfDate(year) => {
            let v = apply_matrix(&transpose(&precession_matrix(year)), pos.to_unit_vector());
            Equatorial::from_unit_vector(v, Epoch::J2000)
        }
    };
    match to {
        Epoch::J2000 => at_j2000,
        Epoch::OfDate(year) => {
            let v = apply_matrix(&precession_matrix(year), at_j2000.to_unit_vector());
            Equatorial::from_unit_vector(v, to)
        }
    }
}

/// IAU 1976 precession matrix taking a J2000 unit vector to epoch-of-`year`.
///
/// `P = R3(-z) · R2(θ) · R3(-ζ)` with the accumulated angles (T = 0 form, since
/// the reference epoch is always J2000).
fn precession_matrix(year: f64) -> [[f64; 3]; 3] {
    let t = (year - 2000.0) / 100.0; // Julian centuries from J2000
    let arcsec = |a: f64| a * (RAD_PER_DEG / 3600.0);
    let zeta = arcsec(2306.2181 * t + 0.301_88 * t * t + 0.017_998 * t * t * t);
    let z = arcsec(2306.2181 * t + 1.094_68 * t * t + 0.018_203 * t * t * t);
    let theta = arcsec(2004.3109 * t - 0.426_65 * t * t - 0.041_833 * t * t * t);
    mat_mul(&mat_mul(&rot_z(-z), &rot_y(theta)), &rot_z(-zeta))
}

fn rot_z(phi: f64) -> [[f64; 3]; 3] {
    let (s, c) = phi.sin_cos();
    [[c, s, 0.0], [-s, c, 0.0], [0.0, 0.0, 1.0]]
}
fn rot_y(phi: f64) -> [[f64; 3]; 3] {
    let (s, c) = phi.sin_cos();
    [[c, 0.0, -s], [0.0, 1.0, 0.0], [s, 0.0, c]]
}
fn mat_mul(a: &[[f64; 3]; 3], b: &[[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let mut out = [[0.0; 3]; 3];
    for (i, row) in out.iter_mut().enumerate() {
        for (j, cell) in row.iter_mut().enumerate() {
            *cell = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j];
        }
    }
    out
}
fn transpose(m: &[[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let mut t = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            t[i][j] = m[j][i];
        }
    }
    t
}
pub(crate) fn apply_matrix(m: &[[f64; 3]; 3], v: [f64; 3]) -> [f64; 3] {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64, eps: f64) -> bool {
        (a - b).abs() < eps
    }
    fn eq(ra: f64, dec: f64) -> Equatorial {
        Equatorial::j2000(Angle::from_degrees(ra), Angle::from_degrees(dec)).unwrap()
    }

    #[test]
    fn equatorial_validates_domain() {
        assert!(eq(10.0, 41.0).ra().degrees() > 0.0);
        assert!(matches!(
            Equatorial::j2000(Angle::from_degrees(360.0), Angle::from_degrees(0.0)),
            Err(Error::OutOfRange {
                what: "right ascension",
                ..
            })
        ));
        assert!(matches!(
            Equatorial::j2000(Angle::from_degrees(0.0), Angle::from_degrees(90.1)),
            Err(Error::OutOfRange {
                what: "declination",
                ..
            })
        ));
        assert!(matches!(
            Equatorial::at_epoch(
                Angle::from_degrees(0.0),
                Angle::from_degrees(0.0),
                Epoch::OfDate(f64::NAN)
            ),
            Err(Error::OutOfRange {
                what: "epoch year",
                ..
            })
        ));
    }

    #[test]
    fn parse_ra_is_hours_dec_is_degrees() {
        let p = Equatorial::parse_j2000("06:00:00", "06:00:00", ParseMode::Strict).unwrap();
        assert!(approx(p.ra().degrees(), 90.0, 1e-9));
        assert!(approx(p.dec().degrees(), 6.0, 1e-9));
    }

    #[test]
    fn separation_known_cases() {
        let m31 = eq(10.6847, 41.2688);
        assert!(separation(m31, m31).arcseconds() < 1e-6);
        let (a, b) = (eq(100.0, 0.0), eq(101.0, 0.0));
        assert!(approx(separation(a, b).degrees(), 1.0, 1e-9));
        let (c, d) = (eq(100.0, 60.0), eq(101.0, 60.0));
        assert!(approx(separation(c, d).degrees(), 0.5, 1e-3));
        let m110 = eq(10.0921, 41.6853);
        assert!((0.4..0.9).contains(&separation(m31, m110).degrees()));
    }

    #[test]
    fn position_angle_cardinal_directions() {
        let c = eq(180.0, 0.0);
        // Due north (+dec) → PA 0; due east (+ra) → PA 90.
        assert!(approx(
            position_angle(c, eq(180.0, 1.0)).degrees(),
            0.0,
            1e-6
        ));
        assert!(approx(
            position_angle(c, eq(181.0, 0.0)).degrees(),
            90.0,
            1e-6
        ));
        assert!(approx(
            position_angle(c, eq(180.0, -1.0)).degrees(),
            180.0,
            1e-6
        ));
        assert!(approx(
            position_angle(c, eq(179.0, 0.0)).degrees(),
            270.0,
            1e-6
        ));
    }

    #[test]
    fn offset_round_trip() {
        let from = eq(10.6847, 41.2688);
        let to = eq(10.0921, 41.6853);
        let off = tangent_offset(from, to);
        let back = apply_offset(from, off);
        assert!(
            separation(to, back).arcseconds() < 1e-3,
            "drift {}",
            separation(to, back).arcseconds()
        );
    }

    #[test]
    fn offset_across_ra_wrap() {
        let from = eq(359.5, 10.0);
        let to = eq(0.5, 10.2);
        let off = tangent_offset(from, to);
        assert!(off.east.degrees() > 0.0, "east across wrap");
        let back = apply_offset(from, off);
        assert!(separation(to, back).arcseconds() < 1e-3);
    }

    #[test]
    fn zero_offset_is_identity() {
        let p = eq(50.0, -30.0);
        let off = TangentOffset {
            east: Angle::from_degrees(0.0),
            north: Angle::from_degrees(0.0),
        };
        assert_eq!(apply_offset(p, off), p);
    }

    #[test]
    fn precession_identity_and_round_trip() {
        let p = eq(45.0, 20.0);
        assert_eq!(precess(p, Epoch::J2000), p);
        let to_date = precess(p, Epoch::OfDate(2050.0));
        assert_eq!(to_date.epoch(), Epoch::OfDate(2050.0));
        let back = precess(to_date, Epoch::J2000);
        assert!(separation(p, back).arcseconds() < 1e-6);
    }

    #[test]
    fn precession_rate_matches_iau() {
        let p = eq(0.0, 0.0);
        let d = precess(p, Epoch::OfDate(2100.0));
        assert!(
            approx(d.dec().arcseconds(), 2004.31, 2.0),
            "dec shift {}",
            d.dec().arcseconds()
        );
        let d26 = precess(p, Epoch::OfDate(2026.0));
        let shift = separation(p, d26).arcminutes();
        assert!((5.0..30.0).contains(&shift), "26yr shift {shift} arcmin");
    }
}
