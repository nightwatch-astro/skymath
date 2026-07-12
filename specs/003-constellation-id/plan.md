# Implementation Plan: skymath v0.3 — Constellation identification

**Input**: [spec.md](spec.md) · **Research**: [research.md](research.md) (R19–R24)

## Technical context

Additive release on the 0.2.0 API: one new module (`constellation`) plus one generated
data module; no new dependencies, no changes to existing modules beyond `lib.rs`
declarations/re-exports. Rust per `rust-toolchain.toml`, `cargo test` + property tests
as before. Same quality gates as 001/002 (fmt, clippy -D warnings, `missing_docs`,
doc build warning-free, AstroPy vector suite, zero Python at test time).

**Constitution check**: constitution is the unfilled template (no ratified
project-specific gates); the standing 001/002 gates above apply. No violations —
Complexity Tracking not needed.

## Public API (contract)

```rust
// module constellation (re-exported at root)
pub enum Constellation { And, Ant, Aps, /* … 88 unit variants, IAU abbreviation casing … */ Vul }

impl Constellation {
    pub const ALL: [Constellation; 88];          // iteration support
    pub fn abbreviation(self) -> &'static str;   // "UMi"
    pub fn name(self) -> &'static str;           // "Ursa Minor", "Boötes" (IAU spellings)
}
impl fmt::Display for Constellation { /* full Latin name */ }
impl FromStr for Constellation { /* abbreviation, case-insensitive; Err = crate Error */ }

pub fn constellation(coord: Equatorial) -> Constellation;   // total: J2000 or of-date in,
                                                            // exactly one of 88 out
```

`Constellation` derives Debug/Clone/Copy/PartialEq/Eq/Hash + serde behind the feature
(derived form = variant identifier = IAU abbreviation).

## Structure

- `src/constellation.rs` — `Constellation` enum, names/abbreviations, `Display`/
  `FromStr`, `constellation()` (precess to B1875.0 via existing `precess()` + R22 table
  walk), provenance module docs; inline tests: known objects (M31, Polaris, σ Octantis,
  both Serpens regions), poles, RA wrap, and direct table-walk unit tests pinning the
  half-open convention in B1875 space.
- `src/constellation_data.rs` — generated 357-record `const` table (do-not-edit header
  naming the generator); `pub(crate)`.
- `scripts/gen_constellation_table.py` — R19 generator (CDS VI/42, AstroPy-bundled copy
  as offline fallback), kept for reproducibility.
- Tests: AstroPy `constellation` section per R24 (`scripts/gen_astropy_vectors.py` +
  `tests/astropy_vectors.rs`, 100% agreement); `tests/properties.rs` — abbreviation ↔
  variant round-trip over `ALL`, case-insensitive parse, all-88-reachable via the
  table, near-wrap consistency; `tests/serde_feature.rs` — serde round-trip +
  abbreviation wire form.
- Docs/provenance: NOTICE paragraph (Roman 1987 / ADC VI/42 data), README bullet,
  DECISIONS.md build-log entry; example `plan_night.rs` gains a constellation line.

## Risks

- Table generation source availability: CDS endpoint may be unreachable — mitigated by
  the AstroPy-bundled byte-identical fallback (R19); the committed generated file means
  generation is a one-time event anyway.
- Name-spelling drift (e.g. "Boötes" diacritic) between our list and the oracle —
  caught structurally: vectors compare full-name strings for all 88 (R24).
- Near-boundary oracle disagreement (R21) — eliminated at generation time by the ±5″
  stability filter; our own boundary convention is pinned by direct table-walk tests
  instead.
