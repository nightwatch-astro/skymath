# Tasks: skymath v0.2 — Sun & Moon ephemerides

**Input**: spec.md, plan.md, research.md (R13–R18)

## Phase 1: Foundational

- [ ] T001 `src/observer.rs`: extract a `pub(crate)` moving-body crossing helper —
  iterate the analytic fixed-body solution against a position callback (3 iterations),
  anchored on transit or anti-transit; fixed-body public API unchanged
- [ ] T002 `src/lib.rs`: declare `sun` and `moon` modules, root re-exports per plan

## Phase 2: User Story 1 — Twilight (P1)

- [ ] T003 [US1] (fresh, Meeus 25) `sun_position` in `src/sun.rs` + inline Meeus 25.a test
- [ ] T004 [US1] (fresh) `Twilight`, `TwilightOutcome`, `twilight()` on the moving-body
  helper anchored at solar anti-transit; serde derives behind the feature
- [ ] T005 [US1] Tests: Leiden 2026-10-15 astronomical dusk/dawn sanity (bracket midnight,
  dusk after sunset), midsummer `NeverDark`, polar-winter `AlwaysDark` in
  `tests/known_values.rs`; latitude-sweep typed-outcome property in `tests/properties.rs`

## Phase 3: User Story 2 — Lunar position & separation (P1)

- [ ] T006 [US2] (fresh, Meeus 47 tables) geocentric `moon_position` + `moon_distance_km`
  in `src/moon.rs` + inline Meeus 47.a pinned test (λ, β, Δ, π)
- [ ] T007 [US2] (fresh, Meeus 40) `moon_position_topocentric` (WGS-84 site coords,
  elevation term) + parallax-magnitude inline test
- [ ] T008 [US2] (fresh) `lunar_separation` (topocentric, target precessed of-date) and
  `moon_crossings` on the moving-body helper
- [ ] T009 [US2] Integration tests in `tests/known_values.rs`: lunar separation vs a
  hand-checked case; moonrise/set window ordering and altitude-at-crossing ≈ threshold

## Phase 4: User Story 3 — Illumination & avoidance (P2)

- [ ] T010 [US3] (fresh, Meeus 48) `moon_phase_angle` + `moon_illumination` + inline 48.a
  pinned test
- [ ] T011 [US3] (fresh) `moon_avoidance_lorentzian` + exact S(0)=S, S(H)=S/2 tests;
  illumination-bounds and avoidance-decay properties in `tests/properties.rs`

## Phase 5: Polish & cross-cutting

- [ ] T012 Extend `scripts/gen_astropy_vectors.py`: `get_sun`, `get_body("moon")`
  geocentric + topocentric, astroplan twilight instants, `moon_illumination`,
  moonrise/moonset; regenerate `tests/data/astropy_vectors.json`
- [ ] T013 Extend `tests/astropy_vectors.rs` with the sun/moon/twilight/illumination
  sections at FR tolerances
- [ ] T014 Serde round-trips for `Twilight`/`TwilightOutcome` in `tests/serde_feature.rs`;
  example `plan_night.rs` gains darkness window + lunar separation + illumination lines
- [ ] T015 Docs: module provenance headers, README bullets, DECISIONS.md build log;
  accuracy-claim audit (every documented tolerance test-pinned); full gate; update spec
  artifacts if implementation deviates

## Dependencies

- T001 → T004/T008; T003 → T004 and T010; T006 → T007 → T008; T006+T003 → T010 → T011;
  Phase 5 last. Meeus examples use TD instants — tests feed them as UTC per R18.
