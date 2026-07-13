# skymath

[![CI](https://github.com/nightwatch-astro/skymath/actions/workflows/ci.yml/badge.svg)](https://github.com/nightwatch-astro/skymath/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/skymath.svg)](https://crates.io/crates/skymath)
[![docs.rs](https://img.shields.io/docsrs/skymath)](https://docs.rs/skymath)

Rust library of planning-grade astronomy math for astrophotography tooling.

[docs.rs](https://docs.rs/skymath) · [Guide](https://docs.rs/skymath/latest/skymath/guide/index.html)

- **Angles** — typed [`Angle`](https://docs.rs/skymath/latest/skymath/struct.Angle.html)
  (degrees, radians, hours, arcminutes, arcseconds), normalization helpers,
  exact conversion constants.
- **Equatorial coordinates** — validated RA/Dec ([`Equatorial`](https://docs.rs/skymath/latest/skymath/struct.Equatorial.html))
  with [`epoch`](https://docs.rs/skymath/latest/skymath/enum.Epoch.html) (J2000 or of-date);
  sexagesimal parsing ([`parse_ra`](https://docs.rs/skymath/latest/skymath/fn.parse_ra.html),
  [`parse_dec`](https://docs.rs/skymath/latest/skymath/fn.parse_dec.html)) in strict and
  lenient modes, and sexagesimal formatting
  ([`format_ra`](https://docs.rs/skymath/latest/skymath/fn.format_ra.html),
  [`format_dec`](https://docs.rs/skymath/latest/skymath/fn.format_dec.html)).
- **Spherical geometry** — great-circle
  [`separation`](https://docs.rs/skymath/latest/skymath/fn.separation.html),
  [`position_angle`](https://docs.rs/skymath/latest/skymath/fn.position_angle.html),
  tangent-plane offsets ([`tangent_offset`](https://docs.rs/skymath/latest/skymath/fn.tangent_offset.html))
  and their inverse ([`apply_offset`](https://docs.rs/skymath/latest/skymath/fn.apply_offset.html)).
- **Precession** — IAU-1976 conversion between J2000 and equinox-of-date via
  [`precess`](https://docs.rs/skymath/latest/skymath/fn.precess.html).
- **Coordinate frames** — equatorial ↔
  [galactic](https://docs.rs/skymath/latest/skymath/struct.Galactic.html)
  ([`to_galactic`](https://docs.rs/skymath/latest/skymath/fn.to_galactic.html),
  [`from_galactic`](https://docs.rs/skymath/latest/skymath/fn.from_galactic.html)) and
  equatorial ↔ [ecliptic](https://docs.rs/skymath/latest/skymath/struct.Ecliptic.html)
  ([`to_ecliptic`](https://docs.rs/skymath/latest/skymath/fn.to_ecliptic.html),
  [`from_ecliptic`](https://docs.rs/skymath/latest/skymath/fn.from_ecliptic.html)).
- **Time** — MJD/JD ↔ calendar conversions
  ([`datetime_to_mjd`](https://docs.rs/skymath/latest/skymath/fn.datetime_to_mjd.html),
  [`mjd_to_datetime`](https://docs.rs/skymath/latest/skymath/fn.mjd_to_datetime.html)),
  MJD ↔ JD ([`jd_to_mjd`](https://docs.rs/skymath/latest/skymath/fn.jd_to_mjd.html),
  [`mjd_to_jd`](https://docs.rs/skymath/latest/skymath/fn.mjd_to_jd.html)),
  FITS `DATE-OBS` parsing and formatting
  ([`parse_date_obs`](https://docs.rs/skymath/latest/skymath/fn.parse_date_obs.html),
  [`format_date_obs`](https://docs.rs/skymath/latest/skymath/fn.format_date_obs.html)),
  Julian epoch from a date ([`julian_epoch_of`](https://docs.rs/skymath/latest/skymath/fn.julian_epoch_of.html)),
  Greenwich and local sidereal time
  ([`gmst`](https://docs.rs/skymath/latest/skymath/fn.gmst.html),
  [`lst`](https://docs.rs/skymath/latest/skymath/fn.lst.html)).
- **Observer-local quantities** — observer
  [`Location`](https://docs.rs/skymath/latest/skymath/struct.Location.html) (with sexagesimal
  parsing via [`Location::parse`](https://docs.rs/skymath/latest/skymath/struct.Location.html#method.parse)),
  [`hour_angle`](https://docs.rs/skymath/latest/skymath/fn.hour_angle.html),
  [alt-azimuth transforms](https://docs.rs/skymath/latest/skymath/fn.alt_az.html),
  [`airmass`](https://docs.rs/skymath/latest/skymath/fn.airmass.html), atmospheric refraction
  ([`refraction_apparent_to_true`](https://docs.rs/skymath/latest/skymath/fn.refraction_apparent_to_true.html),
  [`refraction_true_to_apparent`](https://docs.rs/skymath/latest/skymath/fn.refraction_true_to_apparent.html)),
  [`parallactic_angle`](https://docs.rs/skymath/latest/skymath/fn.parallactic_angle.html), and
  [`transit`](https://docs.rs/skymath/latest/skymath/fn.transit.html) /
  [altitude-crossing](https://docs.rs/skymath/latest/skymath/fn.altitude_crossings.html) times.
- **Sun & Moon** — solar
  ([`sun_position`](https://docs.rs/skymath/latest/skymath/fn.sun_position.html)) and lunar
  positions ([`moon_position`](https://docs.rs/skymath/latest/skymath/fn.moon_position.html),
  geocentric and [topocentric](https://docs.rs/skymath/latest/skymath/fn.moon_position_topocentric.html);
  [`moon_distance_km`](https://docs.rs/skymath/latest/skymath/fn.moon_distance_km.html)),
  [twilight](https://docs.rs/skymath/latest/skymath/fn.twilight.html) times (civil / nautical /
  astronomical, with typed polar-night and midnight-sun outcomes in
  [`TwilightOutcome`](https://docs.rs/skymath/latest/skymath/enum.TwilightOutcome.html)),
  [moonrise/set](https://docs.rs/skymath/latest/skymath/fn.moon_crossings.html),
  [lunar separation](https://docs.rs/skymath/latest/skymath/fn.lunar_separation.html) from a
  target, Moon [illumination](https://docs.rs/skymath/latest/skymath/fn.moon_illumination.html)
  and [phase angle](https://docs.rs/skymath/latest/skymath/fn.moon_phase_angle.html), and the
  [moon-avoidance Lorentzian criterion](https://docs.rs/skymath/latest/skymath/fn.moon_avoidance_lorentzian.html).
- **Constellations** — which of the 88 IAU
  [`Constellation`](https://docs.rs/skymath/latest/skymath/enum.Constellation.html)s
  [contains a coordinate](https://docs.rs/skymath/latest/skymath/fn.constellation.html)
  (Roman 1987 boundary table at B1875.0), as a typed value with official
  abbreviations ("UMi") and Latin names ("Ursa Minor", "Boötes").

Precision is planning-grade (≈1 arcminute) throughout: suitable for framing,
scheduling, and session planning, not for telescope pointing or astrometry.
Apparent-place corrections (nutation, aberration, proper motion) are out of
scope by design.

## Install

```sh
cargo add skymath time
```

The `time` crate supplies the `OffsetDateTime` instants every ephemeris
function takes.

## Usage

```rust
use skymath::{alt_az, altitude_crossings, separation, Angle, CrossingOutcome,
              Equatorial, Location, ParseMode};
use time::OffsetDateTime;

fn main() -> skymath::Result<()> {
    let site = Location::parse("+52 05 32", "+004 18 27", 6.0)?;
    let m31 = Equatorial::parse_j2000("00:42:44.3", "+41:16:09", ParseMode::Strict)?;

    let now = OffsetDateTime::now_utc();
    let h = alt_az(m31, now, &site);
    println!("M31: alt {:.1}°, az {:.1}°", h.altitude.degrees(), h.azimuth.degrees());

    match altitude_crossings(m31, Angle::from_degrees(30.0), now, &site) {
        CrossingOutcome::Crosses { rise, set } => println!("above 30°: {rise} → {set}"),
        outcome => println!("{outcome:?}"),
    }
    Ok(())
}
```

`cargo run --example plan_night` walks the full planning flow (site + target
parsing, constellation lookup, precession to tonight, sidereal time, airmass,
parallactic angle, transit and window, twilight, and Moon separation and
illumination). See the [guide](docs/guide.md) for a task-by-task walkthrough,
or [docs.rs](https://docs.rs/skymath) for the full API reference.

Instants are `time` crate types; functions taking an `OffsetDateTime` fold the
offset in internally, so passing local civil time cannot skew results.

## Configuration

| Feature | Type | Default | Effect |
|---|---|---|---|
| `serde` | Cargo feature | off | Derives `Serialize`/`Deserialize` on all public types. |

## License

Licensed under the [Apache License, Version 2.0](https://github.com/nightwatch-astro/skymath/blob/main/LICENSE).
