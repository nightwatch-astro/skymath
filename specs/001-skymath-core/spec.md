# Feature Specification: skymath v0.1 core

**Feature Branch**: `001-skymath-core`

**Created**: 2026-07-11

**Status**: Draft

**Input**: User description: "skymath v0.1 core — planning-grade astronomy math foundation crate consolidating the coordinate, time, and observer math currently duplicated across target-match, fits-header, and alm, plus the observer-local tier (sidereal time, alt-az, airmass, transit) that exists nowhere in the family's Rust code today."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Shared coordinate and angle primitives (Priority: P1)

A crate maintainer in the product family (target-match, fits-header, simbad-resolver, alm's
metadata crates) replaces their locally duplicated angle/coordinate code with one shared
implementation: typed angles, validated equatorial coordinates carrying their epoch,
sexagesimal parsing (strict and lenient) and formatting, angular separation, position angle,
tangent-plane offsets (forward and inverse), and precession between epochs.

**Why this priority**: This is the consolidation that motivated the crate — the same
primitives exist in at least three divergent copies today (one with a known rounded-constant
drift, one with a silent misparse, one with a display truncation bug). Everything else in the
crate builds on these types.

**Independent Test**: Extractable and testable on its own — the migrated test suites from the
donor crate must pass unchanged, and a downstream crate can parse a sexagesimal position,
precess it, and measure separations with no other part of skymath present.

**Acceptance Scenarios**:

1. **Given** the RA string `"00:42:44.3"` and Dec string `"+41:16:09"`, **When** parsed
   strictly as J2000, **Then** the resulting coordinate equals the decimal form within
   0.1 arcsecond, and formatting it back yields an equivalent sexagesimal string
   (round-trip within formatting precision).
2. **Given** the lenient parse mode and the input `"10 xx 30"`, **When** parsed, **Then**
   the result is an error — never a coordinate that silently ignored the corrupt token.
3. **Given** the lenient parse mode and the input `"10 30"` (missing seconds), **When**
   parsed as RA, **Then** the result is 10h30m exactly (missing fields default to zero).
4. **Given** M31 and M110 J2000 positions, **When** separation and position angle are
   computed, **Then** they match published values within 1 arcminute / 0.5 degree.
5. **Given** a J2000 coordinate and a target epoch of 2026.5, **When** precessed to the
   epoch of date and back, **Then** the round trip returns to the original within
   1 arcsecond.
6. **Given** a coordinate and an East/North tangent-plane offset, **When** the offset is
   applied and then recovered, **Then** the recovered offset matches the input within
   planning-grade tolerance (round-trip property).
7. **Given** a Dec of `-00 30 00`, **When** parsed, **Then** the sign is preserved
   (−0.5°), and **When** formatted, **Then** the leading sign is present.

---

### User Story 2 - Time scales and sidereal time (Priority: P2)

A tool that reads observation timestamps (FITS `DATE-OBS`, MJD values) converts between
calendar datetimes, Julian dates, and Modified Julian Dates; derives a coordinate epoch from
an observation date; and obtains Greenwich or local mean sidereal time for any instant and
longitude.

**Why this priority**: Time conversions are the second existing duplication (fits-header and
alm metadata both carry copies), and sidereal time is the prerequisite for every observer-local
quantity in User Story 3.

**Independent Test**: Testable with nothing but timestamps: known MJD/calendar pairs, the
Meeus worked example for GMST, and epoch derivation for a known date.

**Acceptance Scenarios**:

1. **Given** the FITS string `"2026-07-11T21:30:00"`, **When** parsed, **Then** the calendar
   datetime is exact, and converting to MJD and back is lossless to the millisecond.
2. **Given** 1987-04-10 19:21:00 UTC (Meeus example 12.b), **When** GMST is computed,
   **Then** the result is 8.582524ʰ within 0.1 second of time.
3. **Given** an observation instant carrying any UTC offset, **When** sidereal time is
   computed, **Then** the result is identical to the same instant expressed in UTC
   (offset handling is automatic, not the caller's job).
4. **Given** an observation date of 2026-07-11, **When** a coordinate epoch is derived,
   **Then** the Julian epoch is 2026.52 within 0.01 year.

---

### User Story 3 - Observer-local quantities (Priority: P3)

A session-planning tool holds an observer location (latitude/longitude/elevation, possibly
parsed from FITS `SITELAT`/`SITELONG` sexagesimal strings) and, for a target and an instant,
obtains: hour angle, altitude and azimuth, airmass, an optional refraction correction,
parallactic angle, the target's transit time, and the times at which the target crosses a
given altitude threshold (e.g. "when does it clear 30°").

**Why this priority**: This tier exists nowhere in the family's Rust code today (verified by
inventory); it unblocks backend scheduling/planning features and is the crate's main new
capability. It depends on User Stories 1 and 2.

**Independent Test**: Validated against reference vectors from professional tooling
(AstroPy-derived test vectors inherited from the ported implementation) and published worked
examples, independently of any consumer.

**Acceptance Scenarios**:

1. **Given** a known location, instant, and target, **When** altitude/azimuth are computed,
   **Then** they agree with the AstroPy-derived reference vectors within 1 arcminute
   (unrefracted).
2. **Given** an altitude of 45°, **When** airmass is computed, **Then** it agrees with
   published tables within 1%.
3. **Given** an altitude near the horizon, **When** the refraction correction is requested,
   **Then** it is ~34 arcminutes at 0° and ~1 arcminute at 45°, matching the published
   formula behavior; refraction is opt-in and never silently applied.
4. **Given** a circumpolar target from a high-latitude site, **When** altitude-crossing
   times for 30° are requested, **Then** the answer distinguishes "always above",
   "never above", and "crosses at these times" — it never fabricates a crossing.
5. **Given** a target and site, **When** transit time is requested for a night, **Then**
   the returned instant's hour angle is zero within 5 seconds of time.
6. **Given** `SITELAT "+52 05 32"` / `SITELONG "-004 18 27"` strings, **When** a location
   is parsed, **Then** latitude/longitude match the decimal values within 0.1 arcsecond.

---

### User Story 4 - Sky frame conversions (Priority: P4)

A planning tool expresses an equatorial position in galactic coordinates (how close to the
Milky Way plane is this target) or ecliptic coordinates (groundwork for future sun/moon work).

**Why this priority**: Cheap, useful, and a documented prerequisite for the staged v0.2 tier
(sun/moon/twilight) — but nothing in the family blocks on it.

**Independent Test**: Round-trip properties plus reference vectors (galactic pole and centre,
equinox points).

**Acceptance Scenarios**:

1. **Given** the J2000 galactic centre direction, **When** converted to galactic
   coordinates, **Then** l≈0°, b≈0° within 1 arcminute.
2. **Given** any coordinate, **When** converted to galactic or ecliptic and back, **Then**
   the round trip returns within 1 arcsecond.

---

### Edge Cases

- Poles: position angle and azimuth are ill-conditioned at |Dec| = 90° and at zenith;
  functions must return defined values (documented convention) rather than NaN.
- RA wrap: separations, offsets, and normalization across the 0h/24h boundary.
- Negative-zero declination (`-00 30 00`) must preserve sign through parse and format.
- Sexagesimal formatting must carry rounding (59.9996″ rolls the minute; never emits `60`
  in a seconds or minutes field).
- Lenient parsing: corrupt tokens are an error in every mode; only *missing* trailing
  fields are defaulted.
- Altitude-crossing solver: circumpolar ("always above"), never-rises ("never above"),
  and grazing (tangent) cases return typed outcomes, not fabricated times.
- Observer latitude ±90° and longitude ±180° are valid inputs.
- Non-finite inputs (NaN/∞) are rejected at construction boundaries with typed errors.
- Instants far from J2000 (e.g. year 1900 or 2100): precession and sidereal time remain
  planning-grade; documented as such, no silent precision claims.
- Azimuth convention: North = 0°, East = 90° (documented; tests pin it).

## Requirements *(mandatory)*

### Functional Requirements

**Angles & sexagesimal**

- **FR-A1**: The library MUST provide a typed angle representation constructible from and
  convertible to degrees, radians, hours, arcminutes, and arcseconds, with exact
  published conversion constants.
- **FR-A2**: The library MUST provide angle normalization to [0°, 360°) and (−180°, +180°],
  and hour-angle wrapping.
- **FR-A3**: The library MUST parse sexagesimal RA (hours) and Dec (degrees) in a strict
  mode: fully specified fields, colon or space separated, signs honored, domain-validated
  (RA [0h, 24h), Dec [−90°, +90°]).
- **FR-A4**: The library MUST parse sexagesimal in a lenient mode: flexible separators
  (space/colon/tab), missing minutes/seconds default to zero, sign taken from the leading
  token — and MUST return a typed error for any unparseable token (never silently drop).
- **FR-A5**: The library MUST format angles as sexagesimal RA/Dec with configurable
  precision and separator style, with correct rounding carry and preserved sign
  (including negative zero degrees).

**Coordinates & geometry**

- **FR-C1**: The library MUST provide a validated equatorial coordinate (RA/Dec) that
  always carries its epoch (J2000 or a specified Julian epoch); invalid domains are
  construction-time typed errors.
- **FR-C2**: The library MUST compute great-circle angular separation and position angle
  (degrees East of North) between two coordinates.
- **FR-C3**: The library MUST compute tangent-plane offsets (East/North) from one
  coordinate to another, and apply such an offset to a coordinate (inverse), consistent
  with each other (round-trip property).
- **FR-C4**: The library MUST precess coordinates between any two epochs (IAU-1976 model,
  planning grade), infallibly (epoch data is always present by construction).
- **FR-C5**: The library MUST convert equatorial (J2000) to galactic coordinates and back.
- **FR-C6**: The library MUST convert equatorial to ecliptic coordinates (mean obliquity)
  and back.

**Time**

- **FR-T1**: The library MUST convert between calendar datetimes, Julian Date, and
  Modified Julian Date, losslessly to the millisecond within the supported range.
- **FR-T2**: The library MUST parse and format FITS `DATE-OBS`-style datetime strings
  (with and without time part, quoted or bare), returning a naive datetime that the
  caller explicitly bridges to UTC (documented one-call bridge).
- **FR-T3**: The library MUST derive a coordinate epoch (Julian epoch) from an instant.
- **FR-T4**: The library MUST compute Greenwich Mean Sidereal Time and Local Mean Sidereal
  Time for an instant and longitude; instants carry their UTC offset so misuse of local
  time is impossible by construction.

**Observer**

- **FR-O1**: The library MUST provide an observer location (latitude, longitude, elevation)
  constructible from decimal values or sexagesimal strings, domain-validated.
- **FR-O2**: The library MUST compute hour angle, altitude, and azimuth (North=0°,
  East=90°) for a coordinate, instant, and location.
- **FR-O3**: The library MUST compute airmass from altitude using a published formula,
  with a typed error below the validity horizon.
- **FR-O4**: The library MUST provide an atmospheric refraction correction as an explicit
  opt-in (never silently applied), using a published formula, documented as approximate.
- **FR-O5**: The library MUST compute parallactic angle for a coordinate, instant, and
  location.
- **FR-O6**: The library MUST compute transit time and altitude-crossing times for a
  coordinate, location, and time window, returning typed outcomes for always-above /
  never-above / crossing cases.

**Cross-cutting**

- **FR-X1**: The library MUST perform no I/O and hold no catalogue or ephemeris data
  files; it operates purely on caller-supplied values.
- **FR-X2**: The library MUST expose a single typed error covering parse failures,
  domain violations, and solver validity errors; all failure modes are matchable.
- **FR-X3**: All public types MUST offer optional (de)serialization, off by default,
  at zero cost to non-users.
- **FR-X4**: All computed quantities MUST meet the planning-grade contract: within
  1 arcminute of professional references (AstroPy/ERFA-derived vectors) for angular
  outputs, with stricter internal test tolerances where the algorithm is exact; the
  library MUST NOT claim pointing-grade or astrometric precision.
- **FR-X5**: Ported algorithms MUST carry source attribution and license notices
  compatible with the crate license, and their upstream validation vectors MUST be
  retained as tests.

### Key Entities

- **Angle**: a typed angular quantity; constructors/accessors for all supported units.
- **Equatorial**: RA/Dec + epoch; the family-wide coordinate currency.
- **Epoch**: J2000 or a Julian epoch (of-date); always fully specified.
- **Location**: observer latitude/longitude/elevation.
- **Horizontal**: altitude + azimuth result pair.
- **CrossingOutcome**: always-above / never-above / crossings-with-times result of the
  altitude solver.
- **Error**: the single typed error enum.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Every angular output agrees with professional reference values
  (AstroPy/ERFA-derived vectors, Meeus worked examples) within 1 arcminute; GMST within
  0.1 second of time; airmass/refraction within 1% of published tables.
- **SC-002**: All test suites migrated from donor implementations pass unchanged in
  behavior (the extracted code is a relocation, not a rewrite).
- **SC-003**: All ported algorithms' upstream validation vectors pass in this library.
- **SC-004**: Round-trip properties hold under randomized testing: sexagesimal
  parse↔format, offset apply↔recover, frame convert↔invert, precess to↔from, calendar↔MJD.
- **SC-005**: Corrupt sexagesimal input is rejected in 100% of cases in both parse modes;
  no code path returns a coordinate derived from a dropped token.
- **SC-006**: Every public item is documented; documentation builds without warnings;
  every documented accuracy claim is enforced by a test with an explicit tolerance.
- **SC-007**: A consumer can replace each family duplicate (three sexagesimal parsers, two
  MJD converters, three separation implementations) with a call into this library without
  loss of functionality.
- **SC-008**: The full local verification gate (format, lint at deny-warnings, all tests,
  doc build) passes on all three CI platforms.

## Assumptions

- **Locked design decisions** (from the pre-spec grilling; recorded as constraints):
  public repo; single crate with `serde` as the only optional feature; runtime
  dependencies limited to `thiserror` + `time`; fully typed public API with no parallel
  raw-f64 API; ephemeris inputs are offset-carrying datetimes (`OffsetDateTime`); FITS
  date parsing returns naive datetimes with an explicit UTC bridge; lenient parsing
  errors on garbage; public accuracy contract ≤1 arcminute with stricter internal
  tolerances; MSRV 1.74; Apache-2.0.
- **Code provenance**: the angle/coordinate core and its tests are extracted from
  target-match; date/MJD handling from fits-header; alt-az, galactic, airmass,
  refraction, rise/set structure, and location parsing are ported from
  `gaker/astro-math` (MIT OR Apache-2.0, attribution retained) together with its
  AstroPy-validated test vectors; GMST/LST is written fresh against published worked
  examples (their implementation is ERFA-bound and not liftable).
- **Consumers adopt with breaking changes allowed**: everything downstream is pre-1.0
  and greenfield; target-match will consume skymath types directly with no
  compatibility re-exports (hard cut, decided).
- **Out of scope for v0.1** (staged v0.2): sun/moon positions, twilight windows,
  Lorentzian moon-avoidance, constellation lookup. **Permanently out of scope**:
  apparent-place astrometry (nutation/aberration/proper motion), WCS, optics/FOV math
  (stays in target-match).
- Azimuth convention North=0°/East=90°; sexagesimal RA is in hours, Dec in degrees,
  per astronomical convention.
