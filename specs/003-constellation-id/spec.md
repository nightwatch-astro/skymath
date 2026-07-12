# Feature Specification: skymath v0.3 — Constellation identification

**Feature Branch**: `003-constellation-id`

**Created**: 2026-07-12

**Status**: Draft

**Input**: User description: "constellation identification: given an equatorial coordinate,
return which of the 88 IAU constellations contains it, using the Roman (1987) precomputed
boundary table at B1875.0 with precession of the input coordinate to B1875; typed
Constellation value (88 variants) with IAU 3-letter abbreviation and full Latin name" —
the deferred 003 candidate from the ratified v0.1 scope decision (a data-table feature,
not ephemeris math).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Which constellation is my target in? (Priority: P1)

A planner or imaging tool asks which constellation contains a given sky coordinate — for
target metadata, catalogue grouping, filtering ("everything in Orion tonight"), and
display (M31 → Andromeda). The answer is total: every point on the celestial sphere
belongs to exactly one of the 88 IAU constellations, as fixed by the Delporte (1930) IAU
boundaries.

**Why this priority**: it is the entire feature — a single lookup that downstream tools
(target-match, alm) surface as target metadata; nothing else in the spec exists without
it.

**Independent Test**: identification agrees with AstroPy `get_constellation` on a
generated all-sky vector sample (no Python at test time); well-known objects resolve to
their textbook constellations (M31 → Andromeda, Polaris → Ursa Minor, σ Octantis →
Octans).

**Acceptance Scenarios**:

1. **Given** M31 at J2000 00:42:44.3 +41:16:09, **When** the constellation is queried,
   **Then** the result is Andromeda.
2. **Given** the celestial poles (Dec = ±90°, any RA), **Then** the results are Ursa
   Minor (north) and Octans (south).
3. **Given** a generated all-sky sample of coordinates with AstroPy-confirmed
   constellations, **Then** every case agrees.
4. **Given** the Roman (1987) published check positions (e.g. 9ʰ B1950 +65° → Ursa
   Major), **Then** each reproduces the published constellation.
5. **Given** a coordinate in either of Serpens' two disjoint sky regions (Caput and
   Cauda), **Then** both identify as the single constellation Serpens.

### User Story 2 - Typed constellation with names (Priority: P2)

A consumer receives the identification as a typed value covering exactly the 88 IAU
constellations, each carrying its official IAU 3-letter abbreviation ("And") and full
Latin name ("Andromeda", including correct spellings such as "Boötes"), usable for
display, serialization, and round-tripping from stored abbreviations.

**Why this priority**: the typed surface is what makes the lookup consumable downstream
(serialized plans, UI display, catalogue joins), but it has no value without US1.

**Independent Test**: every one of the 88 values round-trips abbreviation → value →
abbreviation; names and abbreviations match the official IAU list exactly.

**Acceptance Scenarios**:

1. **Given** the value for Andromeda, **Then** its abbreviation is "And" and its name is
   "Andromeda".
2. **Given** the stored abbreviation "UMi" (any letter case), **Then** it parses back to
   Ursa Minor; an unknown abbreviation is a typed error, not a fallback.
3. **Given** any of the 88 values, **Then** abbreviation round-trip is lossless and the
   displayed form is the full Latin name.

### Edge Cases

- **Boundary points**: the IAU boundaries partition the sphere; a coordinate exactly on
  a boundary segment resolves deterministically by the documented half-open convention
  of the Roman (1987) table walk (no dual membership, no error).
- **RA wrap**: coordinates at RA = 0ʰ/24ʰ identify correctly on both sides of the wrap.
- **Poles**: Dec = ±90° (where RA is degenerate) resolve without error for any RA value.
- **Input epoch**: both J2000 and epoch-of-date coordinates are accepted; the epoch is
  honoured internally (precession to B1875.0), matching observer-module semantics.
- **Frame fine print**: the original boundaries are defined in the pre-1976 (FK4) B1875
  frame; identification precesses with the crate's existing model from the modern (FK5)
  frame, exactly as AstroPy does. The discrepancy is at arcsecond scale and can only
  matter for coordinates within ~1″ of a boundary — documented, and the oracle agrees
  because it makes the identical approximation.

## Functional Requirements

- **FR-C1**: `constellation(coord)` — total identification: every valid equatorial
  coordinate (J2000 or of-date) maps to exactly one of the 88 IAU constellations via
  precession to B1875.0 and the Roman (1987) precomputed boundary table (Delporte 1930
  boundaries; ADC/CDS catalogue VI/42, 357 zone records).
- **FR-C2**: `Constellation` — typed value with exactly 88 variants; per-variant official
  IAU 3-letter abbreviation and full Latin name (IAU spellings, e.g. "Boötes");
  display uses the full name; parsing from the abbreviation is case-insensitive and
  returns the crate's typed error on unknown input; abbreviation round-trip is lossless.
- **FR-C3**: agreement with AstroPy `get_constellation`: 100% on a generated all-sky
  vector sample of ≥1000 uniformly distributed points that includes at least one point
  in each of the 88 constellations, evaluated with no Python at test time.
- **FR-C4**: the boundary table and name list are embedded in the crate — no runtime
  I/O, no new dependencies; provenance (Roman 1987, ADC VI/42) credited in module docs
  and NOTICE as for previously ported tables; `serde` derives behind the existing
  feature flag.
- **FR-C5**: every documented behavior above is test-pinned: Roman published check
  positions, known-object cases, pole/wrap/boundary edge cases, round-trip and
  all-variants-reachable properties, and the AstroPy vector section.

## Success Criteria

- **SC-001**: The generated AstroPy vector suite gains a constellation section (≥1000
  points, all 88 constellations represented) and passes with 100% agreement, zero Python
  at `cargo test` time.
- **SC-002**: Roman (1987) check positions and textbook object cases (M31, Polaris,
  σ Octantis, both Serpens regions) all reproduce.
- **SC-003**: All 88 typed values are reachable from lookups in the test suite, and all
  88 round-trip abbreviation ↔ value ↔ name against the official IAU list.
- **SC-004**: Full gate green (`just verify`, `cargo test --all-features`,
  `cargo doc --no-deps --all-features` warning-free) with the crate's `missing_docs`
  lint.
- **SC-005**: v0.2 API untouched — additive release (0.3.0), no breaking changes, no new
  dependencies.
