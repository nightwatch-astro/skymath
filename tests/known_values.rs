//! Known-value integration tests with explicit tolerances (SC-001, SC-005).
//!
//! Each block cites its reference. Tolerances here are the *internal* (strict)
//! bounds; the public contract is ≤ 1 arcminute.

use skymath::{
    alt_az, altitude_crossings, gmst, hour_angle, julian_epoch_of, lst, parse_dec, parse_ra,
    position_angle, separation, transit, Angle, CrossingOutcome, Epoch, Equatorial, Location,
    ParseMode, SexaStyle,
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

// ── US3: observer-local quantities ─────────────────────────────────────────────

fn leiden() -> Location {
    // ≈ Leiden Observatory; 52.155° N, 4.485° E.
    Location::parse("+52 09 18", "+4 29 06", 6.0).unwrap()
}

#[test]
fn site_parse_matches_decimal_to_a_tenth_arcsecond() {
    let l = leiden();
    assert!((l.latitude().degrees() - 52.155).abs() < 3e-5);
    assert!((l.longitude().degrees() - 4.485).abs() < 3e-5);
}

#[test]
fn circumpolar_target_is_always_above_the_horizon() {
    // Polaris (δ ≈ +89.26°) from 52° N never sets: min altitude ≈ 51.4°.
    let polaris = eq(37.9546, 89.2641);
    let outcome = altitude_crossings(
        polaris,
        Angle::from_degrees(0.0),
        datetime!(2026-07-11 22:00 UTC),
        &leiden(),
    );
    assert_eq!(outcome, CrossingOutcome::AlwaysAbove);
}

#[test]
fn far_southern_target_never_rises() {
    // δ = −60° from 52° N: max altitude ≈ −22°.
    let outcome = altitude_crossings(
        eq(83.0, -60.0),
        Angle::from_degrees(0.0),
        datetime!(2026-07-11 22:00 UTC),
        &leiden(),
    );
    assert_eq!(outcome, CrossingOutcome::NeverAbove);
}

#[test]
fn crossing_instants_sit_on_the_threshold() {
    // M31 from Leiden crosses a 30° window; the altitude at each crossing
    // instant must equal the threshold to well under an arcminute.
    let m31 = eq(10.6847, 41.2688);
    let site = leiden();
    let night = datetime!(2026-07-11 22:00 UTC);
    let threshold = Angle::from_degrees(30.0);
    let CrossingOutcome::Crosses { rise, set } = altitude_crossings(m31, threshold, night, &site)
    else {
        panic!("expected Crosses");
    };
    assert!(rise < set);
    for instant in [rise, set] {
        let alt = alt_az(m31, instant, &site).altitude;
        assert!(
            (alt.degrees() - 30.0).abs() * 60.0 < 0.5,
            "altitude at crossing: {}°",
            alt.degrees()
        );
    }
    // And the transit between them is above the window.
    let t = transit(m31, night, &site);
    assert!(rise < t && t < set);
    assert!(alt_az(m31, t, &site).altitude.degrees() > 30.0);
}

#[test]
fn grazing_threshold_yields_a_short_window() {
    // Threshold a hair below M31's maximum altitude from Leiden
    // (90 − |φ − δ|): the window must exist and be short.
    let m31 = eq(10.6847, 41.2688);
    let site = leiden();
    let max_alt = 90.0 - (52.155 - 41.2688);
    let outcome = altitude_crossings(
        m31,
        Angle::from_degrees(max_alt - 0.001),
        datetime!(2026-07-11 22:00 UTC),
        &site,
    );
    let CrossingOutcome::Crosses { rise, set } = outcome else {
        panic!("expected a grazing Crosses, got {outcome:?}");
    };
    assert!(set - rise < time::Duration::minutes(30), "{}", set - rise);
}

#[test]
fn transit_hour_angle_within_five_seconds_of_time() {
    // 5 s of time = 0.0209° of hour angle (SC-001 bound for transit).
    let m31 = eq(10.6847, 41.2688);
    let site = leiden();
    let t = transit(m31, datetime!(2026-07-11 22:00 UTC), &site);
    let ha = hour_angle(m31, t, &site).degrees();
    assert!(ha.abs() < 0.021, "residual HA {ha}°");
}

// ── US4: frame conversions ─────────────────────────────────────────────────────

#[test]
fn galactic_centre_maps_to_the_origin() {
    // Sgr A* / the galactic origin, J2000: 17h45m37.224s, −28°56′10.23″.
    let centre =
        Equatorial::parse_j2000("17:45:37.224", "-28:56:10.23", ParseMode::Strict).unwrap();
    let g = skymath::to_galactic(centre);
    let l = g.l.normalized_pm_180();
    assert!(l.arcminutes().abs() < 1.0, "l = {}′", l.arcminutes());
    assert!(g.b.arcminutes().abs() < 1.0, "b = {}′", g.b.arcminutes());
}

#[test]
fn north_galactic_pole_has_latitude_90() {
    let ngp = eq(192.85948, 27.12825);
    let g = skymath::to_galactic(ngp);
    assert!(
        (g.b.degrees() - 90.0).abs() * 3600.0 < 1.0,
        "b = {}°",
        g.b.degrees()
    );
}

#[test]
fn pollux_ecliptic_matches_meeus_13a() {
    // Meeus example 13.a: Pollux (β Gem) at α = 116.328942°, δ = +28.026183°
    // with the J2000 mean obliquity → λ = 113.215630°, β = +6.684170°.
    let pollux = eq(116.328942, 28.026183);
    let e = skymath::to_ecliptic(pollux, datetime!(2000-01-01 12:00 UTC));
    assert!(
        (e.lambda.degrees() - 113.215630).abs() * 3600.0 < 1.0,
        "λ = {}°",
        e.lambda.degrees()
    );
    assert!(
        (e.beta.degrees() - 6.684170).abs() * 3600.0 < 1.0,
        "β = {}°",
        e.beta.degrees()
    );
}
