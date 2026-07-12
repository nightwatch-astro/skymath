# Implementation Plan: skymath v0.1 core

**Branch**: `001-skymath-core` | **Date**: 2026-07-11 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/001-skymath-core/spec.md`

## Summary

One pure-Rust crate consolidating the family's astronomy math: extract the proven
angle/coordinate core from target-match and the date/MJD handling from fits-header;
port alt-az/galactic/airmass/refraction and the AstroPy-validated vectors from
`gaker/astro-math` (MIT OR Apache-2.0, attributed); write GMST/LST fresh against
published worked examples. Fully typed API, `OffsetDateTime` ephemeris inputs,
planning-grade (≤1′) accuracy contract enforced by tolerance-pinned tests.

## Technical Context

**Language/Version**: Rust, MSRV 1.74, edition 2021 (matches target-match)

**Primary Dependencies**: `thiserror` 2 (typed error), `time` 0.3 (datetime/MJD);
optional `serde` 1 (derives, off by default). Dev-only: `proptest` (property tests),
`serde_json` (feature test), `anyhow` (examples only, never public API).

**Storage**: N/A — no I/O, no data files (FR-X1)

**Testing**: `cargo test` — inline unit tests + integration suites (known values with
explicit tolerances, ported AstroPy vectors, proptest round-trips, serde feature gate);
`just verify` = fmt-check + clippy `-D warnings` + tests + doc build

**Target Platform**: platform-independent library; CI on Linux/Windows/macOS

**Project Type**: single library crate

**Performance Goals**: non-goal; pure f64 math, trivially fast for batch planning grids

**Constraints**: planning-grade accuracy contract (≤1′ public, stricter internal
tolerances); no I/O; no unsafe (`unsafe_code = "forbid"`); every public item documented
(`missing_docs = "warn"` + CI `-D warnings`)

**Scale/Scope**: ~6 source modules, ~2.5–3.5k LOC incl. tests; 4 downstream consumers
queued (target-match, fits-header, simbad-resolver/xisf-header, alm)

## Constitution Check

`.specify/memory/constitution.md` is the unfilled template (deliberate — maintainer
declined ratification family-wide; principles live in `AGENTS.md`). Gates evaluated
against `AGENTS.md` + the family conventions instead:

- Pure library, no I/O, no catalogue data — **pass** (FR-X1)
- Typed public error, no `anyhow` in public API — **pass** (FR-X2; `thiserror`)
- Tests + docs mandatory, deny-warnings CI — **pass** (SC-006, SC-008)
- Dependency austerity — **pass** (2 runtime deps, both already family-standard)
- Conventional commits, release-please, trusted publishing — **pass** (already wired)

Re-check after Phase 1 design: no violations introduced. Complexity tracking: empty.

## Project Structure

### Documentation (this feature)

```text
specs/001-skymath-core/
├── spec.md
├── plan.md              # this file
├── research.md          # Phase 0: algorithm & porting decisions
├── data-model.md        # Phase 1: types, invariants, error taxonomy
├── quickstart.md        # Phase 1: build/verify/use validation guide
├── contracts/
│   └── public-api.md    # Phase 1: full public API contract
├── checklists/requirements.md
└── tasks.md             # Phase 2 (/speckit-tasks)
```

### Source Code (repository root)

```text
src/
├── lib.rs           # crate docs, re-exports, doctest example
├── error.rs         # Error enum (thiserror), Result alias
├── angle.rs         # Angle, normalization, sexagesimal parse (strict+lenient) & format
├── coords.rs        # Epoch, Equatorial, separation, position_angle,
│                    # tangent offsets + inverse, precess (IAU-1976)
├── frames.rs        # equatorial ↔ galactic, equatorial ↔ ecliptic
├── time.rs          # MJD/JD ↔ calendar, DATE-OBS parse/format,
│                    # julian_epoch_of, gmst, lst
└── observer.rs      # Location (+sexagesimal parse), hour_angle, alt_az (Horizontal),
                     # airmass, refraction, parallactic_angle,
                     # transit, altitude_crossings (CrossingOutcome)

tests/
├── known_values.rs      # tolerance-pinned reference values (Meeus, published tables)
├── ported_vectors.rs    # AstroPy-derived vectors lifted from astro-math (attributed)
├── properties.rs        # proptest round-trips (SC-004)
├── coordinates_only.rs  # migrated target-match suite (SC-002)
└── serde_feature.rs     # feature-gated derive coverage

examples/
└── plan_night.rs        # end-to-end: parse site+target, precess, alt-az, transit

NOTICE                   # astro-math attribution (Apache-2.0 practice)
```

**Structure Decision**: flat single-crate module layout mirroring target-match (its
`angle`/`optics`/`matcher` precedent); `frames` split from `coords` to keep each file
reviewable. Extraction origin per module is recorded in module-header docs and NOTICE.

## Complexity Tracking

None — no constitution-gate violations to justify.
