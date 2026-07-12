# Research: skymath v0.3 — Constellation identification

Continues the numbering from 001 (R1–R12) and 002 (R13–R18).

## R19 — Boundary-table source and transcription

**Decision**: generate `src/constellation_data.rs` once with a new
`scripts/gen_constellation_table.py`, reading the Roman (1987) precomputed table
(ADC/CDS catalogue VI/42, `data.dat`, 357 zone records) — fetched from CDS, with the
byte-identical copy bundled inside AstroPy (`constellation_data_roman87.dat`) as an
offline fallback. The generated file is committed; the script stays in `scripts/` for
reproducibility (same policy as `gen_astropy_vectors.py`). Record layout preserved
exactly: catalogue order, lower/upper RA in B1875 hours, lower Dec in B1875 degrees,
constellation code. The two Serpens region codes (`SER1`/`SER2`) map to the single
`Ser` variant.

**Rationale**: 357 × 4 fields is the same transcription-risk class as the 002 Meeus
tables, but unlike Meeus there is a machine-readable original — scripting removes the
risk instead of mitigating it, and the AstroPy 100%-agreement section (R24) re-validates
every record end to end anyway.

**Alternatives considered**: hand-porting (rejected: pure risk, no benefit); embedding
the text table and parsing at runtime (rejected: runtime failure modes and startup cost
versus a `const` array; FR-C4 forbids runtime I/O).

## R20 — B1875.0 expressed in the crate's epoch model

**Decision**: precess inputs with the existing IAU-1976 `precess()` to
`Epoch::OfDate(1875.001_392_3)` (a named `pub(crate)` constant).

**Rationale**: B1875.0 is a Besselian epoch: JD 2415020.31352 + (1875 − 1900) ×
365.242198781 = JD 2405889.2585505, which in Julian years is 2000 + (JD − 2451545)/365.25
= 1875.0013923. `Epoch::OfDate` carries Julian years; its documented day-level precision
is ample — a full 0.0014-yr slip moves coordinates by ~0.07″ at the ~50.3″/yr precession
rate, far below the ~1″ boundary caveat already accepted in the spec.

**Alternatives considered**: adding a Besselian-epoch variant to `Epoch` (rejected:
public-API growth for a single internal constant; nothing else in the crate needs
Besselian epochs).

## R21 — Oracle frame fine print (FK4 vs FK5, Lieske vs Capitaine)

**Decision**: accept and document sub-arcsecond frame/model differences from the oracle;
they cannot affect any generated vector because the sampler discards points within a few
arcseconds of a boundary (R24).

**Rationale**: AstroPy's `get_constellation` transforms to FK5 (equinox B1875) and walks
the same Roman table — the same "FK5 stands in for the original FK4" approximation this
crate makes, and AstroPy documents the resulting near-boundary arcsecond caveat.
Within FK5, AstroPy precesses with the Capitaine et al. (2003) matrix while skymath uses
IAU-1976 (Lieske); the models diverge by ≲0.5″ over the 125-year span. Both effects are
orders of magnitude inside the crate's 1′ contract and only observable within ~1″ of a
boundary.

**Alternatives considered**: implementing FK5→FK4 + Newcomb precession for exactness to
Roman's original frame (rejected: new algorithm surface to chase arcseconds the spec
explicitly waives — and it would *diverge* from the AstroPy oracle, which doesn't do
this either).

## R22 — Lookup semantics (Roman's algorithm)

**Decision**: walk the 357 records in catalogue order; return the first record with
`dec ≥ dec_lo && ra_lo ≤ ra < ra_hi` (RA normalized to [0ʰ, 24ʰ) first). This is
Roman's published algorithm verbatim and defines the spec's "documented half-open
convention": a point on an RA upper edge belongs to the adjacent zone, a point exactly
on a Dec lower edge belongs to that zone (first matching record wins), and the final
records cover the south polar cap so the walk is total — no error path.

**Rationale**: the precomputed table is *designed* for exactly this walk (records are
ordered from the north pole southward); any "smarter" spatial index changes tie-break
behavior and buys nothing at 357 records (~sub-µs linear scan).

**Alternatives considered**: binary search on declination bands (rejected: complicates
the first-match tie-break semantics for zero measurable gain at this table size).

## R23 — Enum shape, names, parsing, serde

**Decision**: `Constellation` is a unit enum with 88 variants named by IAU abbreviation
casing (`And`, `UMi`, `CVn`, …). `abbreviation()` returns the official 3-letter form,
`name()` the full Latin name with IAU spelling (including "Boötes"); `Display` prints
the full name; `FromStr` parses abbreviations case-insensitively and returns the crate's
existing error type on unknown input; `Constellation::ALL: [Constellation; 88]` supports
iteration. Serde derives (behind the existing `serde` feature) serialize as the variant
identifier — i.e. exactly the IAU abbreviation, a stable, compact wire form.

**Rationale**: abbreviation-named variants make the derived serde form meaningful for
free, keep match arms readable, and mirror how astronomers write them; the full-name
`Display` covers UI needs. Name spellings are cross-checked end to end because the
AstroPy vectors carry full names (R24).

**Alternatives considered**: full-name variants like `Andromeda` (rejected: derived
serde form would couple the wire format to long names, and `CanesVenatici`-style idents
read worse in match arms); a string-returning API without an enum (rejected: FR-C2
requires a typed, round-trippable value).

## R24 — AstroPy vector strategy for constellations

**Decision**: extend `scripts/gen_astropy_vectors.py` with a `constellation` section
holding J2000 inputs plus AstroPy's full-name answer, built from: (a) ~1200 seeded
uniform sky points; (b) one witness candidate per Roman table record (zone RA midpoint,
slightly above the zone's lower Dec) so every constellation appears — the script asserts
all 88 names are present and fails generation otherwise; (c) curated probes: M31,
Polaris, σ Octantis, one point in each Serpens region, both celestial poles, an RA-wrap
pair (23ʰ59ᵐ59.9ˢ / 0ʰ0ᵐ0.1ˢ), and the Roman-paper check positions (9ʰ +65° → UMa, etc.).
Every sampled/witness point is kept only if AstroPy returns the same constellation for
the point and four ±5″ offsets around it, so no vector sits within the R21 ambiguity
band. Truth values are stored with `short_names=False`; abbreviations are validated
crate-side against the official 88-entry list instead (avoids any oracle quirk for the
split Serpens codes).

**Rationale**: uniform sampling alone leaves small constellations (Crux 68 deg²,
Sagitta, Equuleus) to chance; the per-record witnesses make all-88 coverage structural
(SC-001/SC-003), and the stability filter makes generation deterministic against
sub-arcsecond model differences (R21) without weakening the 100%-agreement claim.

**Alternatives considered**: raising the uniform sample until coverage happens
(rejected: probabilistic, bloats the JSON); asserting agreement only "away from
boundaries" in the Rust test (rejected: weakens FR-C3's clean 100% statement when the
filter can guarantee it at generation time).
