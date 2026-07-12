# Verify Report — 001-skymath-core (mode: requirements)

VERIFY requirements SUMMARY — PASS: all 26 FRs implemented and wired, all 8 SCs met; local gate green (73 tests / 6 suites, clippy clean, docs warning-free). No functional blockers. One attribution inconsistency in `NOTICE` should be fixed before a public merge; a few acceptance tolerances are validated by stronger adjacent tests rather than their literal stated points.

## Verdict lines (machine-parseable)

```
FR-A1 | IMPLEMENTED | Angle unit ctors/accessors + ARCSEC_PER_RADIAN=206264.80624709636; src/angle.rs:39-92,24; test angle_conversions src/angle.rs:374
FR-A2 | IMPLEMENTED | normalized_0_360/pm_180/hours src/angle.rs:96-116; test angle_normalization src/angle.rs:386
FR-A3 | IMPLEMENTED | strict 3-field parse, colon/space, [0,60) fields src/angle.rs:281-291; RA/Dec domain validated at Equatorial (coords.rs:63); test strict_requires_three_fields src/angle.rs:459
FR-A4 | IMPLEMENTED | lenient flexible sep/default/sign + garbage errors src/angle.rs:259-310; tests garbage_errors_in_every_mode src/angle.rs:441, corrupt_tokens_rejected known_values.rs:48
FR-A5 | IMPLEMENTED | format_ra/dec + SexaStyle, rounding carry, sign src/angle.rs:317-364; tests format_carries_rounding src/angle.rs:474, sign_survives_zero_degrees src/angle.rs:466
FR-C1 | IMPLEMENTED | Equatorial carries Epoch, ctor domain-validates src/coords.rs:51-95; test equatorial_validates_domain src/coords.rs:337
FR-C2 | IMPLEMENTED | separation (haversine) coords.rs:169, position_angle E-of-N coords.rs:186; tests position_angle_cardinal src/coords.rs:386 (1e-6), m31_to_m110 known_values.rs:30
FR-C3 | IMPLEMENTED | tangent_offset coords.rs:211 / apply_offset coords.rs:227; tests offset_round_trip coords.rs:412 + proptest properties.rs:69
FR-C4 | IMPLEMENTED | precess IAU-1976, infallible coords.rs:257-289; tests precession_rate_matches_iau coords.rs:455, proptest properties.rs:60
FR-C5 | IMPLEMENTED | to/from_galactic frames.rs:46,64; tests galactic_centre known_values.rs:229, NGP known_values.rs:240, proptest properties.rs:158
FR-C6 | IMPLEMENTED | to/from_ecliptic + mean_obliquity frames.rs:84-127; tests pollux_meeus_13a known_values.rs:251, mean_obliquity frames.rs:135
FR-T1 | IMPLEMENTED | MJD/JD/calendar/julian_date time.rs:35-94; tests mjd_anchors time.rs:271, calendar_mjd_round_trip proptest properties.rs:91 (<5us)
FR-T2 | IMPLEMENTED | parse/format_date_obs naive + assume_utc bridge time.rs:101-179; tests date_only/quoted/invalid_forms time.rs:214-260
FR-T3 | IMPLEMENTED | julian_epoch_of time.rs:185; test julian_epoch_of_mid_july_2026 known_values.rs:102 (<0.01yr)
FR-T4 | IMPLEMENTED | gmst IAU-1982 + lst, offset folded to UTC time.rs:193-205; tests gmst_matches_meeus known_values.rs:81 (+/-0.1s), gmst_offset_invariant proptest properties.rs:104
FR-O1 | IMPLEMENTED | Location::new/parse domain-validated observer.rs:46-134; tests location_validates_domains observer.rs:344, parse_* observer.rs:358-380
FR-O2 | IMPLEMENTED | hour_angle observer.rs:151, alt_az N=0/E=90 observer.rs:157; test vega_from_kitt_peak ported_vectors.rs:20 (+/-0.01deg)
FR-O3 | IMPLEMENTED | airmass Kasten-Young + <-1deg error observer.rs:200-238; test airmass_matches_kasten_young observer.rs:383
FR-O4 | IMPLEMENTED | refraction Bennett/Saemundsson, opt-in observer.rs:210-225; test refraction_matches_published observer.rs:392 (34.5' at 0, 1' at 45)
FR-O5 | IMPLEMENTED | parallactic_angle observer.rs:246; proptest parallactic_angle_tracks_the_meridian properties.rs:124 (q=0 transit, sign)
FR-O6 | IMPLEMENTED | transit + altitude_crossings -> CrossingOutcome observer.rs:257-328; tests circumpolar/never/crosses/graze known_values.rs:146-214
FR-X1 | IMPLEMENTED | no fs/io/include_bytes in src/ (grep empty); deps thiserror/time/serde only
FR-X2 | IMPLEMENTED | single Error{ParseCoord,ParseDate,OutOfRange} non_exhaustive src/error.rs:12; test display_messages error.rs:41
FR-X3 | IMPLEMENTED | serde cfg_attr on all 12 public data types; test serde_feature.rs:18. NOTE: Error has no serde derive (arguably "where sensible")
FR-X4 | IMPLEMENTED | planning-grade <=1' docs + disclaimer lib.rs:7-11; tolerances pinned across suites
FR-X5 | IMPLEMENTED | ported alt-az attributed (NOTICE + observer.rs:3-12) with AstroPy vectors retained ported_vectors.rs. NOTE: NOTICE over-attributes fresh galactic/airmass/refraction as "ported" (see findings)
SC-001 | MET | alt-az 36" , GMST +/-0.1s, airmass ~0.25%, sep <1' — all pinned
SC-002 | MET | migrated coord suites inline + properties.rs/known_values.rs (donor-file deviation documented); donor-equivalence not independently verifiable from this repo
SC-003 | MET | 5 astro-math alt-az vectors pass ported_vectors.rs
SC-004 | MET | 7 round-trip proptests: sexa, offset, galactic, ecliptic, precess, calendar-MJD properties.rs
SC-005 | MET | corrupt tokens rejected both modes src/angle.rs:441, known_values.rs:48
SC-006 | MET | missing_docs=warn + CI -D warnings; doc build clean; documented tolerances pinned
SC-007 | MET | parse_ra/dec (both modes), MJD converters, separation exported lib.rs:37-55; consumer migration external to repo
SC-008 | MET | fmt-check/clippy/test --all-features/doc green locally; CI matrix ubuntu+windows+macos (.github/workflows/ci.yml). Windows/macOS not executed here
```

## Verify Spec Summary
- Spec: 001-skymath-core
- Requirements checked: 26 FR + 8 SC
- Implemented: 26 | Partial: 0 | Missing: 0 | Diverged: 0 | Inconclusive: 0
- Success criteria: 8 met (SC-002/007/008 have cross-repo / cross-platform portions not executable here)

## Findings By Severity

### Must Fix Before Proceeding
- None. No functional gap, missing surface, or failing check blocks merge.

### Should Address (recommended before a public merge)
1. NOTICE over-attribution (FR-X5). `NOTICE` lists "galactic frame rotation, airmass
   (Kasten-Young) and atmospheric refraction (Bennett/Saemundsson) formulas" as ported
   from `gaker/astro-math`, but research.md (2026-07-12 correction), docs/DECISIONS.md
   build-log, frames.rs:3 ("written fresh") and observer.rs:6-12 ("written fresh") all
   state these were written fresh because astro-math 0.2.1 ships no such modules. The
   public attribution file contradicts the crate's own provenance record. Over-crediting
   is license-safe but factually wrong; fix NOTICE to match the corrected provenance
   (only the alt-az transform + its AstroPy vectors are ported).

### Notes (optional test-tightening; accuracy intent already satisfied)
- PA acceptance US1-4 (0.5deg) is not pinned at that value: m31_to_m110 (known_values.rs:40)
  only bounds PA to 300-325deg. Function correctness is proven separately by
  position_angle_cardinal_directions (coords.rs:386) at 1e-6. Consider a tight M31/M110 PA assert.
- Airmass acceptance US3-2 names 45deg within 1%; the test pins 30deg/90deg/0deg
  (observer.rs:385-387). 30deg is a stronger point (X~1.994 held to ~0.25%), so the
  contract is met; the literal 45deg point is untested.
- FR-X3: Error lacks serde derives. The API contract softens to "where sensible", and a
  transient error type is a reasonable exemption, but a literal reading of "all public
  types" includes it.
- parallactic_angle is validated only by structural properties (q=0 at transit, sign
  either side). research.md R7 also cites AstroPy spot-value cross-checks; no absolute-
  magnitude test is present.

## Verification Commands
- `cargo test --all-features`: pass (73 passed, 6 suites: 38 lib + 18 known_values + 5 ported_vectors + 10 properties + 1 serde + 1 doctest)
- `cargo fmt --check`: pass
- `cargo clippy --all-targets --all-features -- -D warnings`: pass (no issues)
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features`: pass (warning-free)
- `grep -rE "std::fs|std::io|include_bytes|read_to_string" src/`: empty (confirms FR-X1)

## Documented deviations confirmed as non-findings (per spawn instructions)
- No tests/coordinates_only.rs (T010): suites live inline + properties.rs/known_values.rs — documented, correct.
- astro-math 0.2.1 lacks refraction/galactic/rise_set: R3/R4/R5 shifted port->fresh — documented in research.md/DECISIONS.md. (The distinct NOTICE inconsistency above is NOT this deviation; it is the failure to reflect the shift in the attribution file.)
