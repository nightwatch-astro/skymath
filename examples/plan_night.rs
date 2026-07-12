//! End-to-end session-planning walkthrough (the quickstart contract):
//! parse a site and M31, precess to tonight's epoch, print sidereal time,
//! alt-azimuth, airmass, parallactic angle, transit, and the 30° window.
//!
//! Run with `cargo run --example plan_night`.

use skymath::{
    airmass, alt_az, altitude_crossings, gmst, julian_epoch_of, lst, parallactic_angle, precess,
    transit, Angle, CrossingOutcome, Equatorial, Location, ParseMode, SexaStyle,
};
use time::OffsetDateTime;

fn main() -> skymath::Result<()> {
    // FITS SITELAT / SITELONG shapes parse directly.
    let site = Location::parse("+52 05 32", "+004 18 27", 6.0)?;
    println!(
        "Site      {:.4}° N, {:.4}° E, {:.0} m",
        site.latitude().degrees(),
        site.longitude().degrees(),
        site.elevation_m()
    );

    let m31 = Equatorial::parse_j2000("00:42:44.3", "+41:16:09", ParseMode::Strict)?;
    println!(
        "M31 J2000 {} {}",
        m31.ra_sexagesimal(SexaStyle::default()),
        m31.dec_sexagesimal(SexaStyle::default())
    );

    let now = OffsetDateTime::now_utc();
    let tonight = julian_epoch_of(now);
    let of_date = precess(m31, tonight);
    println!(
        "M31 {tonight:?} {} {}",
        of_date.ra_sexagesimal(SexaStyle::default()),
        of_date.dec_sexagesimal(SexaStyle::default())
    );

    println!(
        "GMST      {:.5} h\nLST       {:.5} h",
        gmst(now).hours(),
        lst(now, site.longitude()).hours()
    );

    let h = alt_az(of_date, now, &site);
    println!(
        "Alt/Az    {:.2}° / {:.2}°",
        h.altitude.degrees(),
        h.azimuth.degrees()
    );
    match airmass(h.altitude) {
        Ok(x) => println!("Airmass   {x:.3}"),
        Err(_) => println!("Airmass   n/a (below the horizon)"),
    }
    println!(
        "Parallactic {:.1}°",
        parallactic_angle(of_date, now, &site).degrees()
    );

    println!("Transit   {} (UTC)", transit(of_date, now, &site));
    match altitude_crossings(of_date, Angle::from_degrees(30.0), now, &site) {
        CrossingOutcome::AlwaysAbove => println!("30° window: always above"),
        CrossingOutcome::NeverAbove => println!("30° window: never above"),
        CrossingOutcome::Crosses { rise, set } => {
            println!("30° window: {rise} → {set} (UTC)");
        }
    }
    Ok(())
}
