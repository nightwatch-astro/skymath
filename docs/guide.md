# Guide

A task-by-task walkthrough of `skymath`, planning a night on M31 (RA
`00:42:44.3`, Dec `+41:16:09`) from a site at 52°N, 4°E. Each snippet below is
copy-paste runnable on its own. The full sequence — assembled into one
program — is [`examples/plan_night.rs`](../examples/plan_night.rs); run
`cargo run --example plan_night` to see it end to end.

Add the crate first:

```sh
cargo add skymath time
```

## Parse a site and a target

[`Location::parse`](https://docs.rs/skymath/latest/skymath/struct.Location.html#method.parse)
accepts the FITS `SITELAT`/`SITELONG` sexagesimal shape directly.
[`Equatorial::parse_j2000`](https://docs.rs/skymath/latest/skymath/struct.Equatorial.html#method.parse_j2000)
does the same for a catalogue RA/Dec, with
[`ParseMode::Strict`](https://docs.rs/skymath/latest/skymath/enum.ParseMode.html)
requiring all three sexagesimal fields.

```rust
use skymath::{Equatorial, Location, ParseMode, SexaStyle};

fn main() -> skymath::Result<()> {
    let site = Location::parse("+52 05 32", "+004 18 27", 6.0)?;
    let m31 = Equatorial::parse_j2000("00:42:44.3", "+41:16:09", ParseMode::Strict)?;

    println!(
        "Site {:.4}°N {:.4}°E, {:.0} m",
        site.latitude().degrees(),
        site.longitude().degrees(),
        site.elevation_m()
    );
    println!(
        "M31  {} {}",
        m31.ra_sexagesimal(SexaStyle::default()),
        m31.dec_sexagesimal(SexaStyle::default())
    );
    Ok(())
}
```

## Which constellation is it in?

[`constellation`](https://docs.rs/skymath/latest/skymath/fn.constellation.html)
walks the IAU boundary table and returns a typed
[`Constellation`](https://docs.rs/skymath/latest/skymath/enum.Constellation.html).

```rust
use skymath::{constellation, Equatorial, ParseMode};

fn main() -> skymath::Result<()> {
    let m31 = Equatorial::parse_j2000("00:42:44.3", "+41:16:09", ParseMode::Strict)?;
    let con = constellation(m31);
    println!("M31 is in {con} ({})", con.abbreviation());
    Ok(())
}
```

## Precess to tonight

Catalogue coordinates are J2000; observer-local quantities need the position
at the epoch of the observation. [`julian_epoch_of`](https://docs.rs/skymath/latest/skymath/fn.julian_epoch_of.html)
turns an instant into that epoch, and [`precess`](https://docs.rs/skymath/latest/skymath/fn.precess.html)
moves the coordinate there.

```rust
use skymath::{julian_epoch_of, precess, Equatorial, ParseMode, SexaStyle};
use time::OffsetDateTime;

fn main() -> skymath::Result<()> {
    let m31 = Equatorial::parse_j2000("00:42:44.3", "+41:16:09", ParseMode::Strict)?;
    let tonight = julian_epoch_of(OffsetDateTime::now_utc());
    let of_date = precess(m31, tonight);

    println!(
        "M31 {tonight:?}: {} {}",
        of_date.ra_sexagesimal(SexaStyle::default()),
        of_date.dec_sexagesimal(SexaStyle::default())
    );
    Ok(())
}
```

## Sidereal time at the site

[`gmst`](https://docs.rs/skymath/latest/skymath/fn.gmst.html) is
Greenwich Mean Sidereal Time; [`lst`](https://docs.rs/skymath/latest/skymath/fn.lst.html)
adds the site's east-positive longitude.

```rust
use skymath::{gmst, lst, Location};
use time::OffsetDateTime;

fn main() -> skymath::Result<()> {
    let site = Location::parse("+52 05 32", "+004 18 27", 6.0)?;
    let now = OffsetDateTime::now_utc();

    println!("GMST {:.5} h", gmst(now).hours());
    println!("LST  {:.5} h", lst(now, site.longitude()).hours());
    Ok(())
}
```

## Alt/az, airmass, and parallactic angle

[`alt_az`](https://docs.rs/skymath/latest/skymath/fn.alt_az.html) gives the
target's horizontal position; [`airmass`](https://docs.rs/skymath/latest/skymath/fn.airmass.html)
errors once the target is well below the horizon (< −1°) rather than returning
a nonsensical number.
[`parallactic_angle`](https://docs.rs/skymath/latest/skymath/fn.parallactic_angle.html)
is the position angle of the zenith, useful for planning field rotation.

```rust
use skymath::{airmass, alt_az, julian_epoch_of, parallactic_angle, precess, Equatorial, Location, ParseMode};
use time::OffsetDateTime;

fn main() -> skymath::Result<()> {
    let site = Location::parse("+52 05 32", "+004 18 27", 6.0)?;
    let m31 = Equatorial::parse_j2000("00:42:44.3", "+41:16:09", ParseMode::Strict)?;
    let now = OffsetDateTime::now_utc();
    let of_date = precess(m31, julian_epoch_of(now));

    let h = alt_az(of_date, now, &site);
    println!("Alt/Az {:.2}° / {:.2}°", h.altitude.degrees(), h.azimuth.degrees());

    match airmass(h.altitude) {
        Ok(x) => println!("Airmass {x:.3}"),
        Err(_) => println!("Airmass n/a (below the horizon)"),
    }
    println!(
        "Parallactic {:.1}°",
        parallactic_angle(of_date, now, &site).degrees()
    );
    Ok(())
}
```

## Transit and an altitude window

[`transit`](https://docs.rs/skymath/latest/skymath/fn.transit.html) finds the
meridian crossing nearest a given instant.
[`altitude_crossings`](https://docs.rs/skymath/latest/skymath/fn.altitude_crossings.html)
returns a typed [`CrossingOutcome`](https://docs.rs/skymath/latest/skymath/enum.CrossingOutcome.html)
so circumpolar and never-visible targets can't be confused with a normal
rise/set pair.

```rust
use skymath::{altitude_crossings, julian_epoch_of, precess, transit, Angle, CrossingOutcome, Equatorial, Location, ParseMode};
use time::OffsetDateTime;

fn main() -> skymath::Result<()> {
    let site = Location::parse("+52 05 32", "+004 18 27", 6.0)?;
    let m31 = Equatorial::parse_j2000("00:42:44.3", "+41:16:09", ParseMode::Strict)?;
    let now = OffsetDateTime::now_utc();
    let of_date = precess(m31, julian_epoch_of(now));

    println!("Transit {} (UTC)", transit(of_date, now, &site));
    match altitude_crossings(of_date, Angle::from_degrees(30.0), now, &site) {
        CrossingOutcome::AlwaysAbove => println!("30° window: always above"),
        CrossingOutcome::NeverAbove => println!("30° window: never above"),
        CrossingOutcome::Crosses { rise, set } => {
            println!("30° window: {rise} → {set} (UTC)");
        }
    }
    Ok(())
}
```

## Is it dark enough?

[`twilight`](https://docs.rs/skymath/latest/skymath/fn.twilight.html) solves
for dusk and dawn at a chosen [`Twilight`](https://docs.rs/skymath/latest/skymath/enum.Twilight.html)
level, returning a typed [`TwilightOutcome`](https://docs.rs/skymath/latest/skymath/enum.TwilightOutcome.html)
that distinguishes a normal dark window from a summer night that never gets
dark or a polar night that never brightens.

```rust
use skymath::{twilight, Location, Twilight, TwilightOutcome};
use time::OffsetDateTime;

fn main() -> skymath::Result<()> {
    let site = Location::parse("+52 05 32", "+004 18 27", 6.0)?;
    let now = OffsetDateTime::now_utc();

    match twilight(Twilight::Astronomical, now, &site) {
        TwilightOutcome::Night { dusk, dawn } => println!("Dark sky {dusk} → {dawn} (UTC)"),
        TwilightOutcome::NeverDark => println!("Never astronomically dark tonight"),
        TwilightOutcome::AlwaysDark => println!("Dark around the clock"),
    }
    Ok(())
}
```

## How close is the Moon?

[`lunar_separation`](https://docs.rs/skymath/latest/skymath/fn.lunar_separation.html)
corrects for topocentric parallax internally, so the separation is accurate
for the observing site, not just the Earth's centre.
[`moon_illumination`](https://docs.rs/skymath/latest/skymath/fn.moon_illumination.html)
gives the illuminated fraction of the disk.

```rust
use skymath::{lunar_separation, moon_illumination, Equatorial, Location, ParseMode};
use time::OffsetDateTime;

fn main() -> skymath::Result<()> {
    let site = Location::parse("+52 05 32", "+004 18 27", 6.0)?;
    let m31 = Equatorial::parse_j2000("00:42:44.3", "+41:16:09", ParseMode::Strict)?;
    let now = OffsetDateTime::now_utc();

    println!(
        "Moon {:.1}° from M31, {:.0}% illuminated",
        lunar_separation(m31, now, &site).degrees(),
        moon_illumination(now) * 100.0
    );
    Ok(())
}
```
