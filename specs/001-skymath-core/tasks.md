# Tasks: skymath v0.1 core

**Input**: Design documents from `specs/001-skymath-core/`
**Prerequisites**: plan.md, research.md, data-model.md, contracts/public-api.md, quickstart.md

**Organization**: grouped by user story; each story phase is independently verifiable.
Provenance markers: (extract) = target-match/fits-header relocation, (port) = astro-math
with attribution, (fresh) = new code validated against published references.

## Phase 1: Setup

- [x] T001 Add dependencies via cargo CLI (`thiserror`, `time` w/ macros+formatting+parsing; optional `serde` w/ derive; dev: `proptest`, `serde_json`, `anyhow`) and declare the `serde` feature in `Cargo.toml`
- [x] T002 [P] Write `NOTICE` attributing ported modules/vectors to `gaker/astro-math` (MIT OR Apache-2.0)

## Phase 2: Foundational (blocking)

- [x] T003 Implement `src/error.rs`: `Error { ParseCoord, ParseDate, OutOfRange }` (thiserror, non_exhaustive), `Result<T>` alias, display tests
- [x] T004 Wire `src/lib.rs`: module declarations, root re-exports per contract, crate docs skeleton (final docs in Polish)

## Phase 3: User Story 1 — Shared coordinate & angle primitives (P1) 🎯 MVP

**Goal**: the extraction — typed angles, validated equatorial coords, sexagesimal both
modes/directions, separation/PA/offsets, precession.
**Independent test**: migrated target-match suites + new parse-mode tests pass with only
this phase implemented.

- [x] T005 [US1] (extract) `src/angle.rs`: `Angle` type + unit constructors/accessors + operators + exact constants from target-match `src/angle.rs`; add `normalized_0_360/pm_180/hours` (fresh)
- [x] T006 [US1] (fresh+extract) Sexagesimal core in `src/angle.rs`: one tokenizer, `ParseMode::{Strict,Lenient}` (lenient: flexible separators, missing-fields-default, sign-from-lead, **garbage always errors**), `parse_ra`/`parse_dec`; `format_ra`/`format_dec` with `SexaStyle`, rounding carry, sign preservation
- [x] T007 [US1] (extract) `src/coords.rs`: `Epoch`, `Equatorial` (constructors, `parse_j2000`/`parse_at_epoch` w/ mode, accessors, sexagesimal formatting delegating to angle)
- [x] T008 [US1] (extract+fresh) Geometry in `src/coords.rs`: `separation` (haversine), `position_angle`, `tangent_offset` (hoisted polar decomposition), `apply_offset` (fresh inverse, destination-point)
- [x] T009 [US1] (extract) `precess` IAU-1976 in `src/coords.rs`
- [x] T010 [P] [US1] (extract) Migrate target-match coordinate suites into `tests/coordinates_only.rs` + inline unit tests (SC-002)
  — Deviation: no `tests/coordinates_only.rs`; the donor file of that name tests matcher behaviour (out of skymath scope). The coordinate suites landed as inline unit tests in `src/angle.rs`/`src/coords.rs` plus `tests/properties.rs`/`tests/known_values.rs`.
- [x] T011 [P] [US1] (fresh) Proptest round-trips in `tests/properties.rs`: sexagesimal parse↔format, offset apply↔recover, precess to↔from (SC-004)
- [x] T012 [US1] (fresh) Known values in `tests/known_values.rs`: M31/M110 separation+PA, `-00 30 00` sign, `59.9996″` rounding carry, `"10 xx 30"` rejection in both modes (SC-005)

**Checkpoint**: US1 alone = adoptable by fits-header/simbad-resolver/target-match.

## Phase 4: User Story 2 — Time scales & sidereal time (P2)

**Goal**: MJD/JD/calendar, DATE-OBS, epoch-of-date, GMST/LST.
**Independent test**: timestamp-only tests (Meeus GMST, MJD round-trips).
**Depends on**: Foundational + `Epoch` from T007.

- [x] T013 [US2] (extract) `src/time.rs`: MJD↔datetime, JD↔MJD, `julian_date`, `parse_date_obs`/`format_date_obs` from fits-header `dates.rs` (time-crate types preserved)
- [x] T014 [US2] (fresh) `julian_epoch_of`, `gmst` (IAU-1982 polynomial, hours-normalized), `lst(at, longitude_east)` in `src/time.rs`; UTC-offset handling internal
- [x] T015 [P] [US2] (fresh) Time tests: Meeus 12.a/12.b GMST ±0.1 s, calendar↔MJD lossless round-trip proptest, epoch of 2026-07-11 = 2026.52 ± 0.01, offset-invariance (same instant, different offsets → same GMST) in `tests/known_values.rs` + `tests/properties.rs`

## Phase 5: User Story 3 — Observer-local quantities (P3)

**Goal**: Location, hour angle, alt-az, airmass, refraction, parallactic, transit/crossings.
**Depends on**: US1 (types) + US2 (LST).

- [x] T016 [US3] (fresh, lenient-parser reuse) `Location` in `src/observer.rs`: `new` (domain-validated) + `parse` (decimal / sexagesimal SITELAT-SITELONG / hemisphere suffix)
- [x] T017 [US3] (port) `hour_angle`, `alt_az` → `Horizontal` (N=0/E=90) from astro-math `transforms.rs` (chrono→time swap, typed wrap)
- [x] T018 [P] [US3] (port) `airmass` (Kasten–Young 1989, domain error < −1°) and `refraction_apparent_to_true` (Bennett) / `refraction_true_to_apparent` (Sæmundsson) from astro-math
- [x] T019 [US3] (fresh) `parallactic_angle` (atan2 formula; q=0 at transit)
- [x] T020 [US3] (fresh) `transit` + `altitude_crossings` analytic solver → `CrossingOutcome::{AlwaysAbove,NeverAbove,Crosses}` in `src/observer.rs`
- [x] T021 [P] [US3] (port) Lift AstroPy-derived vectors (alt-az; airmass/refraction expectations) into `tests/ported_vectors.rs` with per-vector provenance comments (SC-003)
- [x] T022 [US3] (fresh) Observer known-values + edge cases in `tests/known_values.rs`: circumpolar → AlwaysAbove, never-rises → NeverAbove, grazing, transit hour-angle ≈ 0 ± 5 s, SITELAT/SITELONG parse ± 0.1″, zenith/pole azimuth convention; parallactic sign flip property in `tests/properties.rs`

## Phase 6: User Story 4 — Sky frame conversions (P4)

**Depends on**: US1 (+ obliquity needs Julian centuries from US2).

- [x] T023 [US4] (port+fresh) `src/frames.rs`: `to/from_galactic` (astro-math rotation, J2000 IAU constants) and `to/from_ecliptic` (fresh, mean obliquity ε(T) Meeus 22.2)
- [x] T024 [P] [US4] (fresh) Frames tests: galactic centre l≈0/b≈0 ± 1′, NGP, round-trips ± 1″ (proptest) in `tests/known_values.rs` + `tests/properties.rs`

## Phase 7: Polish & cross-cutting

- [x] T025 [P] Serde derives behind the feature on all public types + `tests/serde_feature.rs` round-trip (FR-X3)
- [x] T026 [P] `examples/plan_night.rs`: parse site + M31, precess to tonight, GMST/LST, alt-az/airmass/parallactic, transit + 30° window (quickstart contract)
- [x] T027 Final `src/lib.rs` crate docs + root doctest; module-header provenance docs on every module; README updated to the real API with usage snippet
- [x] T028 Accuracy-claim audit: every documented tolerance has a matching pinned test (SC-006); full gate `just verify` + `cargo test --all-features` + `cargo doc --no-deps` warning-free (SC-008)
- [x] T029 Update `docs/DECISIONS.md` (build-phase log complete); confirm spec artifacts consistent with implementation

## Dependencies

- Phase 1 → 2 → 3(US1) → 4(US2) → 5(US3); 6(US4) after US1+T014; 7 last.
- US2 needs `Epoch` (T007). US3 needs `lst` (T014). US4 needs Julian centuries (T014).
- [P] tasks within a phase touch different files and may run in any order.

## Implementation strategy

MVP = Phase 1–3 (US1): immediately adoptable by three consumers. Then US2 → US3 →
US4 in priority order, each phase ending green (`just verify`) and committed as its
own conventional commit(s) on `001-skymath-core`.
