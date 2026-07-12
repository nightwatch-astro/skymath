# Decisions, Assumptions & Questions — skymath v0.1

Running log for the autonomous build. **[DECIDED]** = confirmed by maintainer in the
pre-spec grilling · **[DECISION]** = made autonomously during the build ·
**[ASSUMPTION]** = default taken, flagged for review · **[QUESTION]** = open, needs
maintainer input.

## Maintainer-confirmed (grilling, 2026-07-11)

- **[DECIDED] Repo public** under nightwatch-astro.
- **[DECIDED] Hard cut for target-match**: no compatibility re-exports; target-match 0.2
  uses skymath types directly in its signatures ("it's greenfield, just change it").
- **[DECIDED] Single crate**; only optional feature is `serde`; unconditional deps
  `thiserror` + `time` only.
- **[DECIDED] Fully typed public API** — no parallel f64 convenience API; results in
  small typed structs, not tuples.
- **[DECIDED] `OffsetDateTime` for all ephemeris inputs** (misuse of local time made
  structurally impossible); FITS `DATE-OBS` parsing returns `PrimitiveDateTime` with an
  explicit documented UTC bridge.
- **[DECIDED] Lenient sexagesimal parsing errors on garbage tokens, always.** Lenient
  means flexible separators + missing-field defaults, never acceptance of corrupt input.
  (Behavior change vs fits-header/alm metadata-core; flagged in fits-header#6.)
- **[DECIDED] Accuracy contract**: public promise ≤1 arcminute vs AstroPy/ERFA-derived
  references; internal test tolerances stricter where algorithms are exact (rotations
  ~mas, GMST ±0.1 s, airmass/refraction ~1%).
- **[DECIDED] Streamlined SpecKit run**: specify → plan → tasks → implement → verify,
  autonomous, this grilling standing in for the clarify gate, plus a final
  speckit-verify (requirements) pass.
- **[DECIDED] v0.1 scope** = Tier 1 (extraction) + LST/Alt-Az observer tier; sun/moon/
  twilight/Lorentzian-avoidance/constellations staged to v0.2; apparent-place astrometry
  permanently out.

## Build-phase log

- **[DECISION] Feature numbering/branch**: `001-skymath-core`, mirroring the
  target-match convention (spec dir `specs/001-skymath-core`, PR from branch to main).
- **[DECISION] Version bootstrap**: Cargo.toml + release-please manifest start at 0.0.0
  so the first `feat:` release PR cuts v0.1.0 exactly when the implementation lands.
- **[DECISION] speckit-setup script bug worked around**: upstream setup script crashes
  at its summary stage (`FAILED_EXTENSIONS` unbound when zero extensions fail); the two
  remaining documented steps (workflow definitions, spec-status gitignore entry) were
  run directly per the skill's own documentation.
- **[ASSUMPTION] Optional speckit hooks skipped** (tinyspec classify, refine.status,
  worktree.create): feature is clearly not tiny; repo is dedicated so no worktree needed.
- **[ASSUMPTION] APM dependencies unpinned** (mirrors target-match's apm.yml exactly,
  which is also unpinned); pinning is a family-wide hygiene task, not a skymath concern.
- **[DECISION] astro-math donor reality check**: `gaker/astro-math` 0.2.1 (latest
  published) ships no `refraction`, `galactic`, or `rise_set` modules and no 30 KB
  AstroPy vector file — only `transforms`/`sidereal`/`location`/`time`/`projection`.
  The alt-az transform and its AstroPy cross-check vectors were ported as planned;
  airmass (Kasten–Young 1989), refraction (Bennett/Sæmundsson), and the galactic
  rotation were written fresh against published values. research.md carries the
  correction note; NOTICE attribution stands for the ported parts.
- **[DECISION] T010 file-name deviation**: no `tests/coordinates_only.rs` — the donor
  file of that name tests matcher behaviour (target-match scope). The migrated
  coordinate suites live inline in `src/angle.rs`/`src/coords.rs` and in the
  properties/known-values suites.
- **[DECISION] `serde` feature pulls `time/serde-human-readable`** so
  `CrossingOutcome` (which embeds `OffsetDateTime`) serializes as RFC-3339 strings
  in JSON rather than opaque tuples.
- **[DECISION] AstroPy cross-validation via generated pinned vectors** (2026-07-12):
  `scripts/gen_astropy_vectors.py` (run with `uv run --with astropy --with astroplan`)
  bakes AstroPy 8.0.1 / astroplan 0.10.1 outputs into `tests/data/astropy_vectors.json`;
  `tests/astropy_vectors.rs` validates the full public surface against it with no
  Python at test time. Airmass (Kasten–Young) and refraction (Bennett/Sæmundsson)
  have no AstroPy analogue and stay pinned to their published values.
- **[BUG FOUND & FIXED by the AstroPy suite] Observer functions did not precess**:
  hour_angle/alt_az/parallactic/transit/crossings compared J2000 RA against of-date
  sidereal time (~2 s of RA per year, ≈13′ error for J2000 targets in 2026 — outside
  the 1′ contract). All observer entry points now precess the target to the epoch of
  the instant internally. The astro-math donor has the same defect: its "AstroPy
  verified" Kitt Peak vector was actually its own unprecessed output (AstroPy
  disagrees by ~10′); `tests/ported_vectors.rs` now pins genuine AstroPy values and
  NOTICE no longer credits the vectors.
