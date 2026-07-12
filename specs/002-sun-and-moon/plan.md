# Implementation Plan: skymath v0.2 — Sun & Moon ephemerides

**Input**: [spec.md](spec.md) · **Research**: [research.md](research.md) (R13–R18)

## Technical context

Additive release on the 0.1.0 API: two new modules (`sun`, `moon`), one extension of the
observer machinery (moving-body crossings, private helper shared by twilight and Moon
windows). No new dependencies. Same quality gates as 001 (fmt, clippy -D warnings,
`missing_docs`, doc build warning-free, AstroPy vector suite).

## Public API (contract)

```rust
// module sun (re-exported at root)
pub fn sun_position(at: OffsetDateTime) -> Equatorial;         // apparent, epoch of date
pub enum Twilight { Civil, Nautical, Astronomical }            // −6° / −12° / −18°
pub enum TwilightOutcome {
    Night { dusk: OffsetDateTime, dawn: OffsetDateTime },
    NeverDark,   // sun never descends below the threshold that night
    AlwaysDark,  // sun never ascends above it
}
pub fn twilight(kind: Twilight, night_of: OffsetDateTime, site: &Location) -> TwilightOutcome;

// module moon (re-exported at root)
pub fn moon_position(at: OffsetDateTime) -> Equatorial;        // geocentric, epoch of date
pub fn moon_position_topocentric(at: OffsetDateTime, site: &Location) -> Equatorial;
pub fn moon_distance_km(at: OffsetDateTime) -> f64;
pub fn lunar_separation(target: Equatorial, at: OffsetDateTime, site: &Location) -> Angle;
pub fn moon_crossings(threshold: Angle, night_of: OffsetDateTime, site: &Location)
    -> CrossingOutcome;
pub fn moon_phase_angle(at: OffsetDateTime) -> Angle;
pub fn moon_illumination(at: OffsetDateTime) -> f64;           // [0, 1]
pub fn moon_avoidance_lorentzian(separation_at_full: Angle, half_width_days: f64,
    at: OffsetDateTime) -> Angle;
```

`Twilight`/`TwilightOutcome` derive Debug/Clone/Copy/PartialEq + serde behind the feature.

## Structure

- `src/sun.rs` — Meeus 25 solar position; `Twilight`, `TwilightOutcome`, `twilight()`.
- `src/moon.rs` — Meeus 47 series tables + position; ch. 40 topocentric; ch. 48
  illumination; separation, crossings, avoidance.
- `src/observer.rs` — expose a `pub(crate)` moving-body crossing iterator built on the
  existing analytic solver (fixed-body path unchanged).
- Tests: Meeus examples inline in the modules; integration additions to
  `tests/known_values.rs` (twilight/lunar known cases), `tests/properties.rs` (typed
  outcomes across latitude, illumination bounds, avoidance shape), AstroPy vectors
  extended (`scripts/gen_astropy_vectors.py` + `tests/astropy_vectors.rs`), serde
  round-trip for the new enums in `tests/serde_feature.rs`.

## Risks

- Meeus 47 table transcription: 60+60 coefficient rows — mitigated by the 47.a pinned
  example (any transcription slip shows up at the arcsecond level) and the independent
  AstroPy cross-check.
- Moving-body convergence at extreme latitudes: mitigated by the typed outcomes and the
  latitude sweep property test.
