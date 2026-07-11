# Data Model: skymath v0.1 core

Phase 1 output. Types, invariants, and the error taxonomy. Full signatures live in
[contracts/public-api.md](contracts/public-api.md).

## Types

### `Angle` (Copy)
Typed angular quantity, f64-backed (representation private; extracted from
target-match unchanged).
- Constructors: `from_degrees`, `from_radians`, `from_hours`, `from_arcminutes`,
  `from_arcseconds` (infallible — any finite f64 is an angle).
- Accessors: `degrees()`, `radians()`, `hours()`, `arcminutes()`, `arcseconds()`.
- Normalization: `normalized_0_360()`, `normalized_pm_180()`, `normalized_hours()`
  (→ [0h, 24h)).
- Invariant: none beyond finiteness at *use* boundaries; domain checks live on the
  types that embed angles (Equatorial, Location).

### `Epoch` (Copy)
`J2000 | OfDate(f64 /* Julian epoch, e.g. 2026.52 */)`.
- Invariant: the year is always present and finite (validated where constructed) —
  "of-date without a date" is unrepresentable; precession is therefore infallible
  (FR-C4; inherited target-match design, ratified).

### `Equatorial` (Copy)
`{ ra: Angle, dec: Angle, epoch: Epoch }` — the family-wide coordinate currency.
- Invariants (constructor-enforced, `Error::OutOfRange` on violation): RA ∈ [0°, 360°),
  Dec ∈ [−90°, +90°], all finite.
- Construction: `j2000(ra, dec)`, `at_epoch(ra, dec, epoch)`, `parse_j2000(ra_str,
  dec_str, ParseMode)`, `parse_at_epoch(...)`.
- Formatting: `ra_sexagesimal(SexaStyle)`, `dec_sexagesimal(SexaStyle)` — rounding
  carries, sign preserved (incl. −0°).

### `ParseMode` (Copy)
`Strict | Lenient`.
- Strict: all fields present, `:`/space separated, domains enforced.
- Lenient: separators space/colon/tab, missing minutes/seconds default 0, sign from
  leading token; **any unparseable token → `Error::ParseCoord`** (FR-A4).

### `SexaStyle` (Copy)
Formatting control: separator (`Colons | Spaces`), seconds decimal places.
Defaults mirror target-match output (`HH:MM:SS.sss` / `±DD:MM:SS.ss`).

### `Location` (Copy)
`{ latitude: Angle, longitude: Angle /* east-positive */, elevation_m: f64 }`.
- Invariants: lat ∈ [−90°, +90°], lon ∈ [−180°, +180°], elevation finite.
- Construction: `new(lat, lon, elevation_m)` (typed), `parse(lat_str, lon_str,
  elevation_m)` — decimal, sexagesimal (FITS `SITELAT`/`SITELONG`), or
  hemisphere-suffixed (`"52.09 N"`).

### `Horizontal` (Copy)
`{ altitude: Angle, azimuth: Angle }` — azimuth North=0°, East=90°, ∈ [0°, 360°).

### `CrossingOutcome`
`AlwaysAbove | NeverAbove | Crosses { rise: OffsetDateTime, set: OffsetDateTime }` —
result of the altitude-crossing solver (FR-O6); grazing (|cos H₀| = 1) reported as
`Crosses` with rise == set.

## Error taxonomy

Single enum `Error` (`thiserror`, `#[non_exhaustive]`), `Result<T>` alias:

| Variant | Carries | Raised by |
|---|---|---|
| `ParseCoord(String)` | offending input | sexagesimal/coordinate/location parsing (both modes) |
| `ParseDate(String)` | offending input | `DATE-OBS` parsing |
| `OutOfRange { what: &'static str, value: f64 }` | quantity + value | domain violations: RA/Dec/lat/lon construction, non-finite input, airmass below validity altitude |

Matching target-match's shape (`ParseCoord`/`OutOfRange` retained; `ParseDate` new;
`InvalidOptics` stays behind in target-match). Everything not listed is infallible by
construction (precession, separation, alt-az on valid types).

## Relationships

- `Equatorial` embeds `Epoch`; `precess` maps `Equatorial → Equatorial` (infallible).
- `observer` functions take `(Equatorial, OffsetDateTime, &Location)` → typed results;
  all convert the instant to UTC internally (FR-T4 misuse-proofing).
- `frames`/geometry functions are total on valid `Equatorial` values.
- `serde` feature derives Serialize/Deserialize on: Angle, Epoch, Equatorial,
  Location, Horizontal, CrossingOutcome, ParseMode, SexaStyle (FR-X3).
