# Public API Contract: skymath v0.1

Everything below is re-exported from the crate root unless noted. All functions are
infallible on already-validated types; `Result` appears only at parse/construction/
domain boundaries (see data-model error taxonomy).

## Module `angle`

```rust
pub struct Angle;                       // Copy + PartialEq + PartialOrd + Debug
impl Angle {
    pub const fn from_degrees(d: f64) -> Self;
    pub const fn from_radians(r: f64) -> Self;
    pub fn from_hours(h: f64) -> Self;
    pub fn from_arcminutes(m: f64) -> Self;
    pub fn from_arcseconds(s: f64) -> Self;
    pub fn degrees(self) -> f64;  pub fn radians(self) -> f64;  pub fn hours(self) -> f64;
    pub fn arcminutes(self) -> f64;  pub fn arcseconds(self) -> f64;
    pub fn normalized_0_360(self) -> Self;   // [0°, 360°)
    pub fn normalized_pm_180(self) -> Self;  // (−180°, +180°]
    pub fn normalized_hours(self) -> Self;   // [0h, 24h)
}
// + Add/Sub/Neg/Mul<f64>/Div<f64> operator impls
pub const ARCSEC_PER_RADIAN: f64;       // exact 206264.806…

pub enum ParseMode { Strict, Lenient }
pub struct SexaStyle { pub separator: Separator, pub seconds_places: u8 }
pub enum Separator { Colons, Spaces }
impl Default for SexaStyle;             // Colons; 3 places RA context, 2 Dec context

pub fn parse_ra(s: &str, mode: ParseMode) -> Result<Angle>;   // hours-based
pub fn parse_dec(s: &str, mode: ParseMode) -> Result<Angle>;  // degrees-based, signed
pub fn format_ra(a: Angle, style: SexaStyle) -> String;       // rounding carry, no ":60"
pub fn format_dec(a: Angle, style: SexaStyle) -> String;      // sign always present
```

## Module `coords`

```rust
pub enum Epoch { J2000, OfDate(f64) }   // Julian epoch year
pub struct Equatorial { /* ra, dec, epoch — private */ }
impl Equatorial {
    pub fn j2000(ra: Angle, dec: Angle) -> Result<Self>;
    pub fn at_epoch(ra: Angle, dec: Angle, epoch: Epoch) -> Result<Self>;
    pub fn parse_j2000(ra: &str, dec: &str, mode: ParseMode) -> Result<Self>;
    pub fn parse_at_epoch(ra: &str, dec: &str, epoch: Epoch, mode: ParseMode) -> Result<Self>;
    pub fn ra(self) -> Angle;  pub fn dec(self) -> Angle;  pub fn epoch(self) -> Epoch;
    pub fn ra_sexagesimal(self, style: SexaStyle) -> String;
    pub fn dec_sexagesimal(self, style: SexaStyle) -> String;
}

pub fn separation(a: Equatorial, b: Equatorial) -> Angle;          // haversine
pub fn position_angle(from: Equatorial, to: Equatorial) -> Angle;  // E of N
pub struct TangentOffset { pub east: Angle, pub north: Angle }
pub fn tangent_offset(from: Equatorial, to: Equatorial) -> TangentOffset;
pub fn apply_offset(from: Equatorial, offset: TangentOffset) -> Equatorial;
pub fn precess(pos: Equatorial, to: Epoch) -> Equatorial;          // IAU-1976, infallible
```

## Module `frames`

```rust
pub struct Galactic { pub l: Angle, pub b: Angle }
pub struct Ecliptic { pub lambda: Angle, pub beta: Angle }
pub fn to_galactic(eq: Equatorial) -> Galactic;        // J2000 IAU rotation
pub fn from_galactic(g: Galactic) -> Equatorial;       // returns J2000
pub fn to_ecliptic(eq: Equatorial, at: OffsetDateTime) -> Ecliptic;   // mean obliquity ε(T)
pub fn from_ecliptic(e: Ecliptic, at: OffsetDateTime) -> Equatorial;
```

## Module `time`

```rust
pub fn mjd_to_datetime(mjd: f64) -> Result<PrimitiveDateTime>;
pub fn datetime_to_mjd(dt: PrimitiveDateTime) -> f64;
pub fn jd_to_mjd(jd: f64) -> f64;  pub fn mjd_to_jd(mjd: f64) -> f64;
pub fn julian_date(at: OffsetDateTime) -> f64;

pub fn parse_date_obs(s: &str) -> Result<PrimitiveDateTime>;  // FITS DATE-OBS shapes
pub fn format_date_obs(dt: PrimitiveDateTime) -> String;
// documented bridge: parse_date_obs(s)?.assume_utc() → OffsetDateTime

pub fn julian_epoch_of(at: OffsetDateTime) -> Epoch;   // Epoch::OfDate(2026.52…)
pub fn gmst(at: OffsetDateTime) -> Angle;              // hours-normalized; IAU-1982
pub fn lst(at: OffsetDateTime, longitude_east: Angle) -> Angle;
```

## Module `observer`

```rust
pub struct Location { /* latitude, longitude (east+), elevation_m — private */ }
impl Location {
    pub fn new(latitude: Angle, longitude: Angle, elevation_m: f64) -> Result<Self>;
    pub fn parse(lat: &str, lon: &str, elevation_m: f64) -> Result<Self>;
    pub fn latitude(self) -> Angle;  pub fn longitude(self) -> Angle;
    pub fn elevation_m(self) -> f64;
}

pub struct Horizontal { pub altitude: Angle, pub azimuth: Angle }  // N=0°, E=90°
pub fn hour_angle(target: Equatorial, at: OffsetDateTime, site: &Location) -> Angle;
pub fn alt_az(target: Equatorial, at: OffsetDateTime, site: &Location) -> Horizontal;
pub fn airmass(altitude: Angle) -> Result<f64>;                    // Kasten–Young 1989
pub fn refraction_apparent_to_true(apparent_alt: Angle) -> Result<Angle>;  // Bennett
pub fn refraction_true_to_apparent(true_alt: Angle) -> Result<Angle>;      // Sæmundsson
pub fn parallactic_angle(target: Equatorial, at: OffsetDateTime, site: &Location) -> Angle;

pub enum CrossingOutcome {
    AlwaysAbove, NeverAbove,
    Crosses { rise: OffsetDateTime, set: OffsetDateTime },
}
pub fn transit(target: Equatorial, near: OffsetDateTime, site: &Location) -> OffsetDateTime;
pub fn altitude_crossings(target: Equatorial, threshold: Angle,
                          night_of: OffsetDateTime, site: &Location) -> CrossingOutcome;
```

## Module `error`

```rust
#[non_exhaustive]
pub enum Error {
    ParseCoord(String),
    ParseDate(String),
    OutOfRange { what: &'static str, value: f64 },
}
pub type Result<T> = core::result::Result<T, Error>;
```

## Contract notes

- Ephemeris functions take `OffsetDateTime` and convert to UTC internally (decided);
  passing local time cannot produce wrong sidereal answers.
- Precession, separation, PA, offsets, frames, alt-az, parallactic: **infallible** on
  valid inputs (invalid states unrepresentable) — same philosophy target-match ratified.
- All public types: `Debug + Clone + Copy + PartialEq` where sensible; `serde` derives
  behind the feature.
- Accuracy contract per docs: ≤1′ vs professional references; each documented claim is
  tolerance-pinned in tests (SC-001/SC-006).
