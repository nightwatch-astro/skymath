# Quickstart: skymath v0.1 core

Validation guide — proves the feature works end-to-end. See
[contracts/public-api.md](contracts/public-api.md) for signatures and
[data-model.md](data-model.md) for types.

## Prerequisites

- Rust ≥ 1.74 (pinned via `rust-toolchain.toml`), optionally `just`.

## Full local gate

```sh
just verify          # fmt-check + clippy -D warnings + all tests + doc build
cargo test --all-features   # includes the serde feature suite
cargo doc --no-deps         # must be warning-free (missing_docs enforced)
```

Expected: all suites green — migrated target-match coordinate tests, ported
AstroPy vectors, known-value tests (Meeus/published tables, tolerance-pinned),
proptest round-trips.

## End-to-end example

```sh
cargo run --example plan_night
```

Expected output (values within documented tolerances):

- Parses site `+52 05 32 / +004 18 27` (FITS SITELAT/SITELONG style) and target
  M31 `00:42:44.3 +41:16:09` (J2000).
- Precesses the target to the epoch of tonight's date.
- Prints GMST/LST for the chosen instant (cross-checkable against Meeus 12.b for
  the pinned test date).
- Prints altitude/azimuth (N=0/E=90), airmass, parallactic angle.
- Prints transit time and the 30°-crossing window, or `always above` /
  `never above` for circumpolar/invisible cases.

## Spot validations (map to acceptance scenarios)

| Check | Command | Expect |
|---|---|---|
| Lenient garbage rejected | unit test `lenient_rejects_garbage` | `Error::ParseCoord` for `"10 xx 30"` |
| GMST Meeus 12.b | known-values suite | 8.5825139ʰ ± 0.1 s at 1987-04-10 19:21 UT |
| Alt-az vs AstroPy | ported-vectors suite | ≤ 1′ unrefracted |
| Sexagesimal round-trip | proptest suite | parse ∘ format = identity |
| Offset round-trip | proptest suite | apply ∘ recover = identity |
| Circumpolar solver | known-values suite | `AlwaysAbove`, no fabricated times |
```
