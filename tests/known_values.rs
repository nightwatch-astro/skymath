//! Known-value integration tests with explicit tolerances (SC-001, SC-005).
//!
//! Each block cites its reference. Tolerances here are the *internal* (strict)
//! bounds; the public contract is ≤ 1 arcminute.

use skymath::{
    gmst, julian_epoch_of, lst, parse_dec, parse_ra, position_angle, separation, Angle, Epoch,
    Equatorial, ParseMode, SexaStyle,
};
use time::macros::datetime;

fn eq(ra: f64, dec: f64) -> Equatorial {
    Equatorial::j2000(Angle::from_degrees(ra), Angle::from_degrees(dec)).unwrap()
}

// ── US1: coordinates, parsing, geometry ────────────────────────────────────────

#[test]
fn m31_parses_identically_in_both_modes() {
    // SIMBAD J2000: M31 at 00:42:44.3 +41:16:09 = (10.6846°, 41.2692°).
    for mode in [ParseMode::Strict, ParseMode::Lenient] {
        let p = Equatorial::parse_j2000("00:42:44.3", "+41:16:09", mode).unwrap();
        assert!((p.ra().degrees() - 10.6846).abs() < 1e-3);
        assert!((p.dec().degrees() - 41.2692).abs() < 1e-3);
    }
}

#[test]
fn m31_to_m110_separation_and_direction() {
    // SIMBAD J2000 positions; separation ≈ 36.5′, direction NW (PA ≈ 313°).
    let m31 = eq(10.6847, 41.2688);
    let m110 = eq(10.0921, 41.6853);
    let sep = separation(m31, m110);
    assert!(
        (sep.arcminutes() - 36.5).abs() < 1.0,
        "sep {} arcmin",
        sep.arcminutes()
    );
    let pa = position_angle(m31, m110);
    assert!(
        (300.0..325.0).contains(&pa.degrees()),
        "pa {}",
        pa.degrees()
    );
}

#[test]
fn corrupt_tokens_rejected_in_all_modes() {
    // SC-005: no code path returns a coordinate derived from a dropped token.
    for mode in [ParseMode::Strict, ParseMode::Lenient] {
        assert!(parse_ra("10 xx 30", mode).is_err());
        assert!(parse_dec("41 -- 09", mode).is_err());
        assert!(parse_dec("abc", mode).is_err());
    }
}

#[test]
fn negative_zero_declination_survives() {
    let d = parse_dec("-00 30 00", ParseMode::Lenient).unwrap();
    assert!((d.degrees() + 0.5).abs() < 1e-9);
    let p = Equatorial::j2000(Angle::from_degrees(0.0), d).unwrap();
    assert!(p
        .dec_sexagesimal(SexaStyle::default())
        .starts_with("-00:30"));
}

#[test]
fn seconds_rounding_carries_into_minutes() {
    let a = Angle::from_degrees(10.0 + 59.0 / 60.0 + 59.9996 / 3600.0);
    assert_eq!(
        skymath::format_dec(a, SexaStyle::default()),
        "+11:00:00.00",
        "59.9996 s must roll the minute, never emit :60"
    );
}

// ── US2: time scales & sidereal time ───────────────────────────────────────────

#[test]
fn gmst_matches_meeus_examples() {
    // Meeus, Astronomical Algorithms 2nd ed., examples 12.a and 12.b
    // (1987-04-10): mean sidereal time 13h10m46.3668s at 0h UT and
    // 8h34m57.0896s at 19:21:00 UT.
    let tol_hours = 0.1 / 3600.0; // ±0.1 s of time (SC-001 bound for GMST)
    let a = gmst(datetime!(1987-04-10 00:00 UTC));
    let expect_a = 13.0 + 10.0 / 60.0 + 46.3668 / 3600.0;
    assert!(
        (a.hours() - expect_a).abs() < tol_hours,
        "12.a: got {} h, want {expect_a} h",
        a.hours()
    );
    let b = gmst(datetime!(1987-04-10 19:21 UTC));
    let expect_b = 8.0 + 34.0 / 60.0 + 57.0896 / 3600.0;
    assert!(
        (b.hours() - expect_b).abs() < tol_hours,
        "12.b: got {} h, want {expect_b} h",
        b.hours()
    );
}

#[test]
fn julian_epoch_of_mid_july_2026() {
    let e = julian_epoch_of(datetime!(2026-07-11 00:00 UTC));
    let Epoch::OfDate(year) = e else {
        panic!("expected OfDate, got {e:?}")
    };
    assert!((year - 2026.52).abs() < 0.01, "epoch {year}");
}

#[test]
fn gmst_is_offset_invariant() {
    // The same instant written in two civil offsets.
    let utc = gmst(datetime!(2026-07-11 18:00 UTC));
    let dubai = gmst(datetime!(2026-07-11 22:00 +04:00));
    assert!((utc.hours() - dubai.hours()).abs() < 1e-9);
}

#[test]
fn lst_adds_east_longitude() {
    let at = datetime!(2026-07-11 18:00 UTC);
    let l = lst(at, Angle::from_degrees(60.0));
    let expect = (gmst(at).hours() + 4.0) % 24.0;
    assert!(
        (l.hours() - expect).abs() < 1e-9,
        "lst {} vs {expect}",
        l.hours()
    );
}
