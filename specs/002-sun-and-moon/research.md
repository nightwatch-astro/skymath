# Research: skymath v0.2 — Sun & Moon ephemerides

Continues the numbering from 001's research log.

## R13 — Solar position algorithm

- **Decision**: Meeus ch. 25 "low accuracy" apparent Sun: mean longitude + equation of
  centre (3 terms), apparent λ corrected for aberration and the Ω nutation term, converted
  through the corrected obliquity (ε + 0.00256° cos Ω). Accuracy ~0.01° (36″).
- **Rationale**: inside the ≤1′ contract with ~20 lines; twilight timing sensitivity is
  ~4 min per solar degree, so 0.01° ≈ 2.4 s of time.
- **Alternatives**: VSOP87 truncations — rejected (hundreds of terms for accuracy the
  contract doesn't claim).
- **Validation**: Meeus example 25.a (1992-10-13: λ☉ = 199.90895°, R = 0.99766 AU);
  AstroPy `get_sun` vectors at ≤1′.

## R14 — Lunar position algorithm

- **Decision**: Meeus ch. 47 (truncated ELP-2000/82: the full 60-term tables 47.A and
  47.B) for geocentric λ, β, Δ; topocentric correction per Meeus ch. 40 using the site's
  geocentric coordinates (WGS-84 flattening, elevation term).
- **Rationale**: ~10″ (λ), ~4″ (β), ~20 km (Δ) — far inside contract; the tables are big
  but mechanical constants. Topocentric matters: lunar horizontal parallax is up to ~61′,
  i.e. the geocentric position alone violates the 1′ contract for any site-relative use.
- **Alternatives**: Brown/ELP full theory — rejected (scale); Schlyter's short series
  (~2′) — rejected: within contract only marginally, and Meeus is the book already cited.
- **Validation**: Meeus example 47.a (1992-04-12T00:00 TD: λ = 133.162655°,
  β = −3.229126°, Δ = 368409.7 km, π = 0.991990°); AstroPy `get_body("moon")` geocentric
  and topocentric vectors at ≤2′ (AstroPy uses a fuller theory + frame subtleties).

## R15 — Twilight & Moon rise/set (moving bodies)

- **Decision**: iterate 001's analytic fixed-body altitude-crossing solution: solve with
  the body's position at an estimate, recompute the position at the solved instant,
  re-solve; 3 iterations (Sun moves ~1°/day → converges in 2; Moon ~13°/day → 3).
  Anchor the solve on the body's *anti-transit* for twilight (night brackets midnight)
  and transit for Moon windows. Typed outcomes preserved (`NeverDark`/`AlwaysDark` for
  twilight; `CrossingOutcome` for the Moon).
- **Rationale**: reuses the proven analytic core; no grid search; the R6 deferral said
  exactly this machinery becomes relevant now.
- **Validation**: astroplan `twilight_evening/morning_*` ±60 s; `moonrise/moonset` ±3 min;
  polar/midsummer typed-outcome property across latitudes.

## R16 — Moon illumination

- **Decision**: Meeus ch. 48 exact form: geocentric elongation ψ from the solar and lunar
  positions, phase angle tan i = R sin ψ / (Δ − R cos ψ), illuminated fraction
  k = (1 + cos i)/2.
- **Validation**: Meeus example 48.a (k = 0.6786); astroplan `moon_illumination` ±1%.

## R17 — Moon-avoidance Lorentzian

- **Decision**: the ACP/Berry moon-avoidance criterion: required separation
  S(d) = S_full / (1 + (d/H)²) with d = days from full Moon, derived from the phase angle
  via the mean synodic elongation rate (12.190749°/day). Pure formula on R16's output —
  policy-shaped but math, ratified into v0.2 scope by the 001 grilling.
- **Validation**: S(0) = S_full, S(H) = S_full/2 exactly; monotone decay property.

## R18 — Time-scale handling (ΔT)

- **Decision**: treat UTC as TD (Terrestrial/Dynamical Time) in the series arguments.
  ΔT ≈ 70 s in the current era displaces the Sun by ~2.9″ (0.04″/s of longitude motion is
  wrong by a factor — the Sun moves 360°/365.25 d ≈ 0.041″/s) and the Moon by ~38″
  (~0.55″/s). Both sit inside the documented claims (Sun ≤1′, Moon ≤2′ vs AstroPy) and
  shift twilight instants by ~3 s and moonrise by well under a minute. Documented per
  function. *(Correction 2026-07-12: the draft of this entry understated the lunar
  effect as 0.04″; the pinned Meeus 47.a test is unaffected because it compares at the
  same numeric Julian date.)*
- **Alternatives**: ΔT polynomial models — rejected as precision theater at this grade.
