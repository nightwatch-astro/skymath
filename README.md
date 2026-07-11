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

Precision is planning-grade (≈1 arcminute) throughout: suitable for framing,
scheduling, and session planning, not for telescope pointing or astrometry.
Apparent-place corrections (nutation, aberration, proper motion) are out of
scope by design.

## License

Licensed under the [Apache License, Version 2.0](LICENSE).
