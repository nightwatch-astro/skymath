# Verify Report — 003 Constellation identification

**Verdict: PASS.** All five functional requirements (FR-C1…FR-C5) are IMPLEMENTED and
all five success criteria (SC-001…SC-005) are met. Full gate is green. No must-fix or
should-address findings; three low-severity notes below.

Branch `003-constellation-id`, HEAD `c577dad`, working tree clean.

## Per-item verdict lines

```
FR-C1 | VERIFIED | constellation() precesses to B1875 + walks 357-record Roman table; total; of-date honoured
FR-C2 | VERIFIED | 88 variants, abbreviation()/name(), Display=name, case-insensitive FromStr→typed Error, lossless round-trip
FR-C3 | VERIFIED | 1576 astropy cases (≥1000), 88 distinct, 100% agreement asserted, include_str! (no Python)
FR-C4 | VERIFIED | const ZONES, no I/O, Cargo.toml unchanged, serde optional, provenance in NOTICE + module docs
FR-C5 | VERIFIED | Roman check / known-object / pole / wrap / half-open / round-trip / all-88 tests present and pass
SC-001| VERIFIED | constellation section ≥1000, all 88, zero Python at test time
SC-002| VERIFIED | Roman 135°/+65°→UMa, M31→And, Polaris→UMi, σ Oct, both Serpens reproduce
SC-003| VERIFIED | all 88 reachable via table walk; all 88 round-trip abbr↔value↔name
SC-004| VERIFIED | just verify green; cargo test --all-features 120 passed; cargo doc -D warnings exit 0
SC-005| VERIFIED | additive only (non_exhaustive Error variant + new module); no signature changes; no new deps
```

## Requirement details

| ID | Status | Evidence | Gap |
|----|--------|----------|-----|
| FR-C1 | IMPLEMENTED | `constellation()` `src/constellation.rs:466-469` precesses via existing `precess()` to `B1875_JULIAN_YEAR=1875.0013923` (`:24`, R20); `lookup_b1875` `:474-484` walks `ZONES` first-match `dec>=dec_lo && ra_lo<=ra<ra_hi` with `ra.rem_euclid(24)`; `ZONES: [...; 357]` in catalogue order `src/constellation_data.rs:13`, total via final `(0,24,-90,Oct)` `:370`. Of-date honoured through `precess()` on the input's stored epoch — `of_date_input_honoured` `src/constellation.rs:548-553`, `constellation_honours_input_epoch` `tests/astropy_vectors.rs:507-522`. | — |
| FR-C2 | IMPLEMENTED | 88 variants `src/constellation.rs:35-212`; `ALL:[_;88]` `:320-409`; `abbreviation()` `:413`, `name()` `:420`; `Display` writes name `:425-429`; `FromStr` case-insensitive `eq_ignore_ascii_case`, `Err=Error::UnknownConstellation` `:431-446`. IAU spellings Boötes/Chamaeleon/Ophiuchus/Piscis Austrinus at `NAMES :235,:248,:285,:293`. Round-trip pinned `tests/properties.rs:263-278`; serde wire form = abbreviation `tests/serde_feature.rs:71-80`. | — |
| FR-C3 | IMPLEMENTED | `tests/data/astropy_vectors.json` `constellation` = 1576 cases, 88 distinct abbreviations (verified via json load). `tests/astropy_vectors.rs:481-505` asserts `len>=1000`, abbr match, full-name match, `seen.len()==88`. `include_str!` `:42` — no Python at `cargo test`. | — |
| FR-C4 | IMPLEMENTED | `const ZONES` generated-file header (names generator, do-not-edit) `src/constellation_data.rs:1-7`; no runtime I/O. `Cargo.toml` unchanged on branch (empty `git diff main...HEAD -- Cargo.toml`); `serde` optional dep `Cargo.toml:20` behind feature `:35`; derive gated `#[cfg_attr(feature="serde",…)]` `src/constellation.rs:34`. Provenance: `NOTICE:20-26` (Roman 1987 / ADC VI/42 / Delporte 1930 / B1875), module docs `src/constellation.rs:1-12`, `docs/DECISIONS.md:77-85`. | — |
| FR-C5 | IMPLEMENTED | Roman check 9ʰ/+65° → UMa present in vector set (case at 135.0/65.0 → UMa) and gen probes `scripts/gen_astropy_vectors.py:374`; known objects `src/constellation.rs:495-521`; poles any-RA `:523-533`; RA wrap `:535-546`; half-open convention direct table-walk `:555-565`; totality sweep `:567-588`; all-88 reachable + round-trip `tests/properties.rs:245-278`. | — |

## Success criteria

| ID | Status | Evidence |
|----|--------|----------|
| SC-001 | MET | 1576 ≥ 1000 cases, all 88 present; 100% agreement asserted `tests/astropy_vectors.rs:485,:504`; zero Python via `include_str!`. |
| SC-002 | MET | Curated probes reproduce: Roman 135°/+65°→UMa, M31→And (10.685/41.269), Polaris→UMi (37.95/89.26), σ Oct, both Serpens (α Ser Caput, η Ser Cauda) `scripts/gen_astropy_vectors.py:367-378`; all pass in vector run. |
| SC-003 | MET | `constellation_all_88_reachable` grid sweep `tests/properties.rs:245-258`; `constellation_abbreviation_round_trips` over `ALL` `:263-278`; full-name pinned end-to-end in vectors. |
| SC-004 | MET | `just verify` (fmt-check + clippy `-D warnings` + test) green; `cargo test --all-features` = 120 passed (7 suites); `RUSTDOCFLAGS=-D warnings cargo doc --no-deps --all-features` exit 0 (`missing_docs` clean). |
| SC-005 | MET | Additive only: `Error::UnknownConstellation` on `#[non_exhaustive]` enum `src/error.rs:11,:24-25`; `src/lib.rs` adds module + re-exports. `git diff main...HEAD` of angle/coords/frames/moon/observer/sun/time = empty (no signature changes). No new deps. |

## Notes (low severity, no action required)

- **Crate version is still `0.2.0`** (`Cargo.toml:3`), not `0.3.0`. The repo uses
  release-please (recent `chore(main): release …` commits); the minor bump is a
  release-automation outcome the `feat:` commit will trigger, not something carried on
  the feature branch. SC-005's "additive release (0.3.0)" intent holds — the API surface
  is additive. Flagged only so the release step is not forgotten.
- **Of-date coverage is thin**: `constellation_of_date` JSON has 1 case; plus one inline
  `of_date_input_honoured` test. FR-C1's of-date path is the shared `precess()` from the
  input's stored epoch — the same code J2000 cases exercise heavily and the precession
  suite validates independently — so risk is low. More of-date witnesses would harden it.
- **Doc-comment ordering claim**: `src/constellation.rs:318-319` says `ALL` is "ordered
  alphabetically by abbreviation", but the actual order is the IAU canonical order
  (e.g. `Aqr` before `Aql`, `Cnc` before `CVn`/`CMa`). Cosmetic only — the parallel
  `ABBREVIATIONS`/`NAMES` arrays are index-aligned to the enum discriminant and every
  variant's abbr/name is round-trip- and oracle-verified, so a misalignment would fail
  the suite. No functional impact.

## Verification commands

- `just verify`: pass (fmt-check, clippy `-D warnings`, `cargo test`)
- `cargo test --all-features`: pass (120 passed, 0 failed, 7 suites)
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features`: pass (exit 0)
- `git diff main...HEAD -- Cargo.toml`: empty (no dependency/version churn on branch)
- `git diff main...HEAD -- src/{angle,coords,frames,moon,observer,sun,time}.rs`: empty (no existing-API changes)
