# Research: skymath v0.1 core

Phase 0 output. Every algorithm/porting choice with rationale and alternatives.
No NEEDS CLARIFICATION markers existed in the spec; this consolidates the technical
research behind each module.

> **Implementation-phase correction (2026-07-12)**: `gaker/astro-math` 0.2.1 (the
> latest published version) contains only `location`, `nutation`, `projection`,
> `sidereal`, `time`, and `transforms` modules. The `refraction.rs`, `galactic.rs`,
> and `rise_set.rs` modules cited below, the four-formula airmass surface, and the
> "30 KB AstroPy test file"/25-format location parser do not exist in it. The
> *decisions* stand unchanged; the provenance shifts: only the alt-azimuth
> transform and its AstroPy cross-check vectors are ports (R2), while airmass (R3),
> refraction (R4), and the galactic rotation (R5) are written fresh and validated
> against published reference values.

## R1 — GMST/LST algorithm

- **Decision**: IAU-1982 GMST polynomial (Meeus, *Astronomical Algorithms* 2nd ed.,
  ch. 12, eq. 12.4) on the UT1≈UTC assumption; LST = GMST + east longitude.
- **Rationale**: ~0.1 s-of-time accuracy over ±1 century — comfortably inside the
  planning-grade contract; 10 lines of code; zero dependencies. UT1−UTC (|ΔUT1| < 0.9 s)
  contributes < 0.9 s of time ≈ 13.5″ of hour angle — inside the 1′ contract; documented.
- **Alternatives**: ERFA IAU-2006 via `erfars` (astro-math's route) — rejected: heavy
  dependency for milliarcsecond accuracy we don't claim. Porting astronomy-engine's
  sidereal — rejected: chrono/JS-shaped, no better at planning grade.
- **Validation**: Meeus examples 12.a/12.b (1987-04-10): GMST 13.1795463ʰ (0 UT) and
  8.5825139ʰ (19:21 UT), tolerance ±0.1 s; cross-checked against astro-math's doctest
  values (ERFA-derived, agree at our tolerance).

## R2 — Alt-azimuth transform

- **Decision**: port astro-math `transforms.rs` (standard spherical triangle via hour
  angle/declination/latitude), swapping chrono→`time` and wrapping in typed API.
  Azimuth convention North=0°, East=90° (matches astro-math and AstroPy AltAz).
- **Rationale**: self-contained pure math (verified no ERFA import); ships with a 30 KB
  AstroPy-validated test file whose vectors we lift.
- **Alternatives**: fresh implementation — rejected: identical math, but we'd forfeit
  the pedigree of their validated edge cases (poles, zenith).

## R3 — Airmass formula

- **Decision**: one blessed function, Kasten–Young (1989); domain error below −1°
  apparent altitude (formula validity edge).
- **Rationale**: the standard choice for planning tools; accurate to ~1% at altitude
  > 5°, well-behaved to the horizon. One function avoids "which of four formulas"
  API noise for a planning-grade crate.
- **Alternatives**: plane-parallel secant (fails < 20°), Pickering 2002 (marginally
  better < 3°, where planning decisions don't live), Young 1994. astro-math ships all
  four; we port the Kasten–Young arm + its tests and note the rest as v0.x additions
  if a consumer asks.

## R4 — Refraction

- **Decision**: Bennett (1982) formula for apparent→true and its Sæmundsson inverse
  for true→apparent, ported from astro-math `refraction.rs`; standard-conditions
  constants documented (1010 hPa, 10 °C); strictly opt-in per FR-O4.
- **Alternatives**: temperature/pressure-parameterized variants — deferred; the
  standard-condition formula is within ~0.1′ of parameterized versions above 15°.

## R5 — Galactic & ecliptic frames

- **Decision**: galactic = fixed J2000 rotation (IAU pole α=192.85948°, δ=27.12825°,
  l of NCP 122.93192°), ported from astro-math `galactic.rs` (dependency-free);
  ecliptic = rotation by mean obliquity ε(T), IAU-1976 polynomial (Meeus 22.2),
  written fresh (~15 lines).
- **Rationale**: both are exact rotations — internal test tolerance ~milliarcsecond;
  round-trip property tests are decisive.
- **Alternatives**: none serious at this grade.

## R6 — Transit & altitude-crossing solver

- **Decision**: analytic solution. Transit: instant where LST = RA (solved directly
  from GMST linearity within a day). Crossings: cos H₀ = (sin h₀ − sin φ sin δ) /
  (cos φ cos δ); |cos H₀| > 1 yields the typed AlwaysAbove/NeverAbove outcomes;
  otherwise rise/set instants = transit ∓ H₀ converted to solar time.
- **Rationale**: for fixed stars (RA/Dec constant over a night) the analytic answer is
  exact at planning grade; no iteration, no convergence edge cases. astro-math's
  iterative `rise_set.rs` handles moving bodies (sun/moon) — that machinery becomes
  relevant in v0.2, not now. Its test cases (circumpolar, never-rises, equator) are
  still lifted as behavioral references.
- **Alternatives**: port the iterative scanner — rejected for v0.1 as unneeded
  complexity; grid-search (alm's TS approach) — rejected: analytic is strictly better.

## R7 — Parallactic angle

- **Decision**: standard formula q = atan2(sin H, tan φ cos δ − sin δ cos H), written
  fresh (astro-math has no parallactic module).
- **Validation**: q = 0 at transit (property), sign flips across meridian (property),
  spot values cross-checked against AstroPy `parallactic_angle` published examples.

## R8 — Observer location parsing

- **Decision**: `Location::parse` accepts decimal degrees or sexagesimal via skymath's
  own lenient parser (FITS `SITELAT "+52 05 32"` shapes), plus explicit
  hemisphere-suffix handling (`"52.09 N"`, `"4.31 W"`). **Trimmed port**: astro-math's
  25-format/30 KB regex parser is *not* ported.
- **Rationale**: the family's real inputs are FITS keywords and UI decimal fields; the
  regex zoo adds a dependency (`regex`) and untestable surface. Its test cases for the
  formats we do support are lifted.
- **Alternatives**: full port — rejected (dependency + scope); decimal-only — rejected
  (SITELAT/SITELONG are sexagesimal in the wild).

## R9 — Tangent-plane offsets

- **Decision**: hoist target-match's polar decomposition (separation + position angle
  → ΔE/ΔN, never dividing by cos of separation); inverse = spherical destination-point
  formula (bearing = position angle, distance = separation), written fresh.
- **Validation**: round-trip proptest (apply ∘ recover = identity within 1 mas at
  planning scales); known values at RA-wrap and near-pole positions.

## R10 — Time representation

- **Decision**: `time` crate. JD/MJD as f64 (µs-level precision near the current
  era — far below planning grade); MJD epoch 1858-11-17T00:00 UTC as the anchor;
  `OffsetDateTime` in, converted to UTC internally; `PrimitiveDateTime` out of
  `DATE-OBS` parsing (FITS strings carry no offset) with `assume_utc()` as the
  documented bridge.
- **Alternatives**: `hifitime` — rejected for v0.1 (leap-second machinery unneeded at
  planning grade; heavy for 6 conversions); two-part JD — rejected (precision theater
  at this grade).

## R11 — Testing strategy

- **Decision**: four integration suites (known values / ported vectors / proptest
  properties / serde gate) + inline unit tests; float comparison via explicit
  per-assertion tolerances (target-match precedent — no `approx` dependency); every
  public accuracy claim in docs carries a test with the same number.
- **Ported-vector provenance**: each lifted vector cites its astro-math source file
  and (where astro-math recorded it) the AstroPy version that produced it.

## R12 — Attribution mechanics

- **Decision**: `NOTICE` file at repo root (Apache-2.0 practice) naming
  `gaker/astro-math` (dual MIT/Apache-2.0) for ported modules + vectors; module-header
  doc comments state per-file provenance ("ported from", "extracted from",
  "written fresh, validated against").
