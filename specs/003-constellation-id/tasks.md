# Tasks: skymath v0.3 — Constellation identification

**Input**: spec.md, plan.md, research.md (R19–R24)

## Phase 1: Foundational

- [x] T001 `src/constellation.rs`: `Constellation` unit enum — 88 variants in IAU
  abbreviation casing (`And` … `Vul`), `ALL: [Constellation; 88]`, `abbreviation()`,
  `name()` (IAU spellings incl. "Boötes"); derives Debug/Clone/Copy/PartialEq/Eq/Hash
- [x] T002 `scripts/gen_constellation_table.py`: fetch ADC/CDS VI/42 `data.dat` (AstroPy
  bundled copy as offline fallback), emit generated `src/constellation_data.rs` — 357
  `const` records (ra_lo/ra_hi B1875 hours, dec_lo B1875 degrees, `Constellation`),
  catalogue order, `SER1`/`SER2` → `Ser`, do-not-edit provenance header; run it and
  commit the output (record count + first/last rows spot-checked against the catalogue)
- [x] T003 `src/lib.rs`: declare `constellation` + `constellation_data` modules, root
  re-exports (`Constellation`, `constellation`)

## Phase 2: User Story 1 — Which constellation is my target in? (P1)

- [x] T004 [US1] `src/constellation.rs`: `B1875` epoch constant (R20),
  R22 table walk (first record with `dec ≥ dec_lo && ra_lo ≤ ra < ra_hi`, RA normalized
  to [0ʰ,24ʰ)), public `constellation(coord)` precessing via existing `precess()`;
  inline unit tests pinning the half-open convention, RA wrap, and both poles directly
  in B1875 table space
- [x] T005 [US1] `src/constellation.rs` inline known-object tests: M31 → And,
  Polaris → UMi, σ Octantis → Oct, one point in each Serpens region → Ser, Dec ±90°
  (any RA), RA-wrap pair 23ʰ59ᵐ59.9ˢ/0ʰ00ᵐ00.1ˢ
- [x] T006 [US1] `scripts/gen_astropy_vectors.py`: add `constellation` section per R24 —
  ~1200 seeded uniform points + one witness per table record + curated probes (M31,
  Polaris, σ Oct, Serpens ×2, poles, wrap pair, Roman-paper checks), ±5″ stability
  filter, assert all 88 names present; regenerate `tests/data/astropy_vectors.json`
- [x] T007 [US1] `tests/astropy_vectors.rs`: constellation section — map AstroPy full
  names to variants, require 100% agreement (FR-C3/SC-001)

## Phase 3: User Story 2 — Typed constellation with names (P2)

- [x] T008 [US2] `src/constellation.rs`: `Display` (full name), `FromStr`
  (case-insensitive abbreviation, crate `Error` on unknown), serde derives behind the
  existing feature (wire form = variant identifier = abbreviation)
- [x] T009 [P] [US2] `tests/properties.rs`: abbreviation ↔ variant round-trip over
  `ALL`, case-insensitive parse property, all-88-reachable through the table walk,
  near-wrap consistency property
- [x] T010 [P] [US2] `tests/serde_feature.rs`: `Constellation` serde round-trip; wire
  form equals the IAU abbreviation

## Phase 4: Polish & cross-cutting

- [x] T011 [P] `examples/plan_night.rs`: constellation line for the target; README
  feature bullet
- [x] T012 Docs & gate: module provenance header, NOTICE paragraph (Roman 1987 /
  ADC VI/42), `docs/DECISIONS.md` build-log entry; accuracy/behavior-claim audit (every
  documented convention test-pinned); full gate (`just verify`,
  `cargo test --all-features`, `cargo doc --no-deps --all-features` warning-free);
  update spec artifacts if implementation deviates

## Dependencies

- T001 → T002 → T003; T003 → T004 → T005/T006; T006 → T007; T001 → T008 →
  T009/T010 (T009, T010 parallel); T011 parallel after T004; T012 last.
- MVP = Phase 1 + Phase 2 (US1): identification working and oracle-validated; US2 adds
  the parse/serde surface on the same enum.
