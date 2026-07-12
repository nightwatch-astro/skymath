# skymath

Rust library of planning-grade astronomy math for astrophotography tooling.

- **Angles** — typed `Angle` (degrees, radians, hours, arcminutes, arcseconds),
  normalization helpers, exact conversion constants.
- **Equatorial coordinates** — validated RA/Dec with epoch (J2000 or of-date);
  sexagesimal parsing in strict and lenient modes, and sexagesimal formatting.
- **Spherical geometry** — great-circle separation, position angle,
  tangent-plane offsets and their inverse (offset applied to a coordinate).
- **Precession** — IAU-1976 conversion between J2000 and equinox-of-date.
- **Coordinate frames** — equatorial ↔ galactic and equatorial ↔ ecliptic.
- **Time** — MJD/JD ↔ calendar conversions, FITS `DATE-OBS` parsing and
  formatting, Julian epoch from a date, Greenwich and local sidereal time.
- **Observer-local quantities** — observer `Location` (with sexagesimal
  parsing), hour angle, alt-azimuth transforms, airmass, atmospheric
  refraction, parallactic angle, and transit / altitude-crossing times.
- **Sun & Moon** — solar and lunar positions (geocentric and topocentric),
  twilight times (civil / nautical / astronomical, with typed polar-night and
  midnight-sun outcomes), moonrise/set, lunar separation from a target, Moon
  illumination and phase angle, and the moon-avoidance Lorentzian criterion.

Precision is planning-grade (≈1 arcminute) throughout: suitable for framing,
scheduling, and session planning, not for telescope pointing or astrometry.
Apparent-place corrections (nutation, aberration, proper motion) are out of
scope by design.

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
parsing, precession to tonight, sidereal time, airmass, parallactic angle,
transit and window). Enable the `serde` feature for `Serialize`/`Deserialize`
derives on all public types.

Instants are `time` crate types; functions taking an `OffsetDateTime` fold the
offset in internally, so passing local civil time cannot skew results.

## License

Licensed under the [Apache License, Version 2.0](LICENSE).
