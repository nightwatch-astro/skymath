# Feature Specification: skymath v0.2 — Sun & Moon ephemerides

**Feature Branch**: `002-sun-and-moon`

**Created**: 2026-07-12

**Status**: Draft

**Input**: User description: "sun and moon ephemerides: solar and lunar positions, lunar
separation, moon illumination, twilight times, moon-avoidance" — the bundle staged to v0.2
by the ratified v0.1 scope decision (constellation identification stays deferred; it is a
data-table feature, not ephemeris math).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Twilight times for session planning (Priority: P1)

A session planner asks when the sky is dark enough to image on a given night at a given
site: civil (−6°), nautical (−12°), and astronomical (−18°) dusk and dawn instants, with
typed outcomes for high-latitude nights where a given darkness level is never reached
(bright summer) or never left (polar winter).

**Why this priority**: darkness bounds every imaging plan; it is the first question any
scheduler asks and it needs only the solar position plus the existing crossing machinery.

**Independent Test**: twilight instants for a mid-latitude site match astroplan within
±60 s; a Leiden midsummer night reports "never astronomically dark"; a polar-winter case
reports always-dark.

**Acceptance Scenarios**:

1. **Given** Leiden on 2026-10-15, **When** astronomical twilight is queried, **Then**
   dusk and dawn instants bracket local midnight and match astroplan within ±60 s.
2. **Given** Leiden on 2026-06-21 (astronomical), **Then** the outcome is the typed
   "never dark enough" variant, not a fabricated instant.
3. **Given** Longyearbyen (78° N) in late December (civil), **Then** the outcome is the
   typed "always dark" variant.

### User Story 2 - Lunar position and separation (Priority: P1)

A planner asks where the Moon is at an instant — geocentric or topocentric (lunar parallax
reaches ~1°, which is far outside the crate's 1′ contract, so the site-corrected position
is the planning-relevant one) — and how far the Moon sits from an imaging target.

**Why this priority**: Moon proximity is the second veto (after darkness) in every target
selection; `lunar_separation` is the family's most requested missing function.

**Independent Test**: geocentric position matches Meeus example 47.a; topocentric and
geocentric positions differ by the expected parallax; separation and moonrise/set match
AstroPy/astroplan at planning tolerances.

**Acceptance Scenarios**:

1. **Given** 1992-04-12T00:00 TD (Meeus 47.a), **When** the geocentric lunar position is
   computed, **Then** λ = 133.162655°, β = −3.229126°, Δ = 368409.7 km within the
   documented truncation tolerance (≤10″ / ≤20 km).
2. **Given** M31 and a site/instant, **Then** `lunar_separation` equals the great-circle
   separation between the topocentric Moon and the target within 2′ of AstroPy.
3. **Given** a site and night, **Then** Moon altitude crossings (rise/set at 0°) match
   astroplan within ±3 min (the Moon moves ~0.5°/h; the solver iterates to convergence).

### User Story 3 - Moon illumination and avoidance (Priority: P2)

A scheduler scores targets by lunar interference: illuminated fraction and phase angle at
an instant, and the classic moon-avoidance Lorentzian (required separation largest at full
Moon, relaxing with lunar age) as a ready-made criterion.

**Why this priority**: converts US2's raw geometry into the decision numbers schedulers
actually use; depends on both solar and lunar positions.

**Independent Test**: illuminated fraction matches Meeus example 48.a (k = 0.6786) and
astroplan `moon_illumination` within 1%; the Lorentzian returns its configured maximum at
full Moon and half of it at the configured half-width in days.

**Acceptance Scenarios**:

1. **Given** 1992-04-12T00:00 TD (Meeus 48.a), **Then** k = 0.68 within 0.01.
2. **Given** a full-Moon instant, **Then** `moon_avoidance_lorentzian(S, H, at)` returns S;
   at H days from full it returns S/2; far from full it approaches 0.

## Functional Requirements

- **FR-S1**: `sun_position(at)` — apparent geocentric solar position (epoch of date),
  documented accuracy ≤ 1′ vs AstroPy `get_sun`.
- **FR-S2**: `twilight(kind, night_of, site)` — Civil/Nautical/Astronomical dusk & dawn
  for the night containing/nearest `night_of`, typed outcome
  (`Night { dusk, dawn } | NeverDark | AlwaysDark`), ±60 s vs astroplan.
- **FR-M1**: `moon_position(at)` — geocentric lunar position (epoch of date) with
  documented truncation accuracy (≤10″ in longitude vs Meeus 47.a; ≤2′ vs AstroPy which
  uses a fuller theory); `moon_distance_km(at)` exposed.
- **FR-M2**: `moon_position_topocentric(at, site)` — parallax-corrected position
  (Meeus ch. 40), ≤2′ vs AstroPy topocentric.
- **FR-M3**: `lunar_separation(target, at, site)` — topocentric Moon↔target great-circle
  separation; target precessed to the epoch of date internally (matching observer-module
  semantics).
- **FR-M4**: `moon_crossings(threshold, night_of, site)` — Moon altitude crossings via
  the moving-body iteration of the analytic solver, ±3 min vs astroplan.
- **FR-I1**: `moon_phase_angle(at)` (Meeus ch. 48, exact tan i form with distances) and
  `moon_illumination(at)` = (1 + cos i)/2 ∈ [0, 1], ±1% vs astroplan.
- **FR-I2**: `moon_avoidance_lorentzian(separation_at_full, half_width_days, at)` —
  required minimum separation S / (1 + (d/H)²) where d = days from full Moon derived from
  the phase angle and the mean elongation rate.
- **FR-X1**: no new dependencies; new public types get `serde` derives behind the existing
  feature; instants are `OffsetDateTime` folded to UTC internally (UT1≈UTC and TD≈UTC at
  planning grade — ΔT ≈ 70 s moves the Moon ~0.04″ in the series argument, negligible).
- **FR-X2**: every documented tolerance above is pinned by a test (Meeus examples for the
  algorithms, generated AstroPy/astroplan vectors for end-to-end validation, property
  tests for ranges/round-trips).

## Success Criteria

- **SC-001**: Meeus 47.a and 48.a reproduce within documented truncation tolerances.
- **SC-002**: AstroPy vector suite extended with sun/moon/twilight/illumination cases; all
  pass at the FR tolerances with no Python at test time.
- **SC-003**: Twilight/moonrise solvers never fabricate instants — polar/never cases are
  typed outcomes (property test across latitudes).
- **SC-004**: Full gate green (`just verify`, `cargo test --all-features`,
  `cargo doc --no-deps --all-features` warning-free) with the crate's `missing_docs` lint.
- **SC-005**: v0.1 API untouched — additive release (0.2.0), no breaking changes.
