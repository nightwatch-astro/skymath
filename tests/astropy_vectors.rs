//! Cross-validation of the public API against AstroPy-generated reference
//! vectors (SC-001/SC-003: the ≤1′ planning-grade contract, checked against
//! the professional reference implementation).
//!
//! The vectors live in `tests/data/astropy_vectors.json`, generated offline by
//! `scripts/gen_astropy_vectors.py` (versions pinned in the file's `meta`);
//! `cargo test` needs no Python. Regenerate with
//! `uv run --with astropy --with astroplan scripts/gen_astropy_vectors.py`.
//!
//! Tolerance rationale:
//! - Frame-independent geometry (separation, position angle, offsets,
//!   precession vs FK5, galactic) uses the same mathematics as AstroPy —
//!   tolerances are arcsecond-level.
//! - Observer-facing quantities (alt-az, hour angle, parallactic, transit,
//!   crossings) omit nutation (~17″), annual aberration (~20″), and ΔUT1
//!   (≤0.9 s ≈ 13″) by design; AstroPy applies the full apparent-place
//!   chain, so tolerances are the public 1′ claim (time-shaped bounds for
//!   transit/crossings).
//! - Not AstroPy-checkable, validated elsewhere: airmass (Kasten–Young has
//!   no AstroPy implementation; pinned to the published K&Y values inline in
//!   `src/observer.rs`) and refraction (AstroPy's ERFA model differs from
//!   Bennett/Sæmundsson; pinned to their published values inline).

use serde_json::Value;
use skymath::{
    alt_az, altitude_crossings, apply_offset, datetime_to_mjd, gmst, hour_angle, julian_date,
    julian_epoch_of, lst, moon_crossings, moon_distance_km, moon_illumination, moon_position,
    moon_position_topocentric, parallactic_angle, parse_dec, parse_ra, position_angle, precess,
    separation, sun_position, tangent_offset, transit, twilight, Angle, CrossingOutcome, Epoch,
    Equatorial, Location, ParseMode, TangentOffset, Twilight, TwilightOutcome,
};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

fn vectors() -> Value {
    serde_json::from_str(include_str!("data/astropy_vectors.json")).unwrap()
}

fn f(v: &Value, key: &str) -> f64 {
    v[key].as_f64().unwrap_or_else(|| panic!("missing {key}"))
}

fn radec(v: &Value) -> Equatorial {
    let (ra, dec) = (v[0].as_f64().unwrap(), v[1].as_f64().unwrap());
    Equatorial::j2000(Angle::from_degrees(ra), Angle::from_degrees(dec)).unwrap()
}

fn site(v: &Value) -> Location {
    Location::new(
        Angle::from_degrees(v[0].as_f64().unwrap()),
        Angle::from_degrees(v[1].as_f64().unwrap()),
        v[2].as_f64().unwrap(),
    )
    .unwrap()
}

fn instant(v: &Value, key: &str) -> OffsetDateTime {
    OffsetDateTime::parse(v[key].as_str().unwrap(), &Rfc3339).unwrap()
}

/// Smallest angular difference between two angles in degrees.
fn wrap_diff_deg(a: f64, b: f64) -> f64 {
    Angle::from_degrees(a - b)
        .normalized_pm_180()
        .degrees()
        .abs()
}

/// Angular distance between two horizontal positions, in arcseconds
/// (azimuth/altitude reinterpreted on a sphere so near-zenith azimuth noise
/// is weighted correctly).
fn horizontal_sep_arcsec(alt1: f64, az1: f64, alt2: f64, az2: f64) -> f64 {
    let a = Equatorial::j2000(
        Angle::from_degrees(az1).normalized_0_360(),
        Angle::from_degrees(alt1),
    )
    .unwrap();
    let b = Equatorial::j2000(
        Angle::from_degrees(az2).normalized_0_360(),
        Angle::from_degrees(alt2),
    )
    .unwrap();
    separation(a, b).arcseconds()
}

#[test]
fn separation_and_position_angle_match_astropy() {
    for v in vectors()["separation_pa"].as_array().unwrap() {
        let (a, b) = (radec(&v["a"]), radec(&v["b"]));
        let sep = separation(a, b).degrees();
        assert!(
            (sep - f(v, "sep_deg")).abs() * 3600.0 < 0.05,
            "sep {sep} vs {}",
            f(v, "sep_deg")
        );
        let pa = position_angle(a, b).degrees();
        assert!(
            wrap_diff_deg(pa, f(v, "pa_deg")) < 0.005,
            "pa {pa} vs {}",
            f(v, "pa_deg")
        );
    }
}

#[test]
fn tangent_offsets_match_spherical_offsets_to() {
    // AstroPy's SkyOffsetFrame and the polar decomposition agree to O(sep³);
    // at frame-scale separations (≤2°) that is well under 5″.
    for v in vectors()["offsets"].as_array().unwrap() {
        let TangentOffset { east, north } = tangent_offset(radec(&v["a"]), radec(&v["b"]));
        assert!(
            (east.degrees() - f(v, "east_deg")).abs() * 3600.0 < 5.0,
            "east {} vs {}",
            east.degrees(),
            f(v, "east_deg")
        );
        assert!(
            (north.degrees() - f(v, "north_deg")).abs() * 3600.0 < 5.0,
            "north {} vs {}",
            north.degrees(),
            f(v, "north_deg")
        );
    }
}

#[test]
fn apply_offset_matches_directional_offset_by() {
    for v in vectors()["apply_offset"].as_array().unwrap() {
        let from = radec(&v["from"]);
        let (sep, pa) = (f(v, "sep_deg"), f(v, "pa_deg"));
        let offset = TangentOffset {
            east: Angle::from_degrees(sep * pa.to_radians().sin()),
            north: Angle::from_degrees(sep * pa.to_radians().cos()),
        };
        let dest = apply_offset(from, offset);
        let drift = separation(dest, radec(&v["to"])).arcseconds();
        assert!(drift < 0.1, "drift {drift}″");
    }
}

#[test]
fn precession_matches_fk5() {
    for v in vectors()["precess"].as_array().unwrap() {
        let p = precess(radec(&v["j2000"]), Epoch::OfDate(f(v, "epoch")));
        let expected = Equatorial::at_epoch(
            Angle::from_degrees(v["of_date"][0].as_f64().unwrap()),
            Angle::from_degrees(v["of_date"][1].as_f64().unwrap()),
            Epoch::OfDate(f(v, "epoch")),
        )
        .unwrap();
        let drift = separation(p, expected).arcseconds();
        assert!(drift < 1.0, "epoch {}: drift {drift}″", f(v, "epoch"));
    }
}

#[test]
fn galactic_matches_astropy() {
    for v in vectors()["galactic"].as_array().unwrap() {
        let g = skymath::to_galactic(radec(&v["eq"]));
        assert!(
            (g.b.degrees() - f(v, "b_deg")).abs() * 3600.0 < 2.0,
            "b {} vs {}",
            g.b.degrees(),
            f(v, "b_deg")
        );
        // Longitude degenerates at the poles; check it away from them.
        if f(v, "b_deg").abs() < 89.99 {
            assert!(
                wrap_diff_deg(g.l.degrees(), f(v, "l_deg")) * 3600.0
                    < 2.0 / f(v, "b_deg").to_radians().cos().abs().max(0.01),
                "l {} vs {}",
                g.l.degrees(),
                f(v, "l_deg")
            );
        }
    }
}

#[test]
fn ecliptic_matches_astropy_within_a_minute() {
    // AstroPy's chain (ICRS → GCRS → mean ecliptic, IAU-2006 models) differs
    // from the classical mean-obliquity rotation at the tens-of-arcsec level.
    for v in vectors()["ecliptic"].as_array().unwrap() {
        let e = skymath::to_ecliptic(radec(&v["eq"]), instant(v, "at"));
        assert!(
            wrap_diff_deg(e.lambda.degrees(), f(v, "lambda_deg")) * 3600.0 < 60.0,
            "λ {} vs {}",
            e.lambda.degrees(),
            f(v, "lambda_deg")
        );
        assert!(
            (e.beta.degrees() - f(v, "beta_deg")).abs() * 3600.0 < 60.0,
            "β {} vs {}",
            e.beta.degrees(),
            f(v, "beta_deg")
        );
    }
}

#[test]
fn gmst_and_lst_match_astropy_iau1982() {
    // Both sides treat the instant as UT1, so this pins the polynomial itself.
    let tol_hours = 0.01 / 3600.0; // 10 ms of time
    for v in vectors()["gmst"].as_array().unwrap() {
        let ours = gmst(instant(v, "at")).hours();
        assert!(
            (ours - f(v, "hours")).abs() < tol_hours,
            "gmst {ours} vs {}",
            f(v, "hours")
        );
    }
    for v in vectors()["lst"].as_array().unwrap() {
        let ours = lst(instant(v, "at"), Angle::from_degrees(f(v, "lon_east_deg"))).hours();
        assert!(
            (ours - f(v, "hours")).abs() < tol_hours,
            "lst {ours} vs {}",
            f(v, "hours")
        );
    }
}

#[test]
fn julian_dates_match_astropy() {
    for v in vectors()["jd"].as_array().unwrap() {
        let at = instant(v, "at");
        assert!((julian_date(at) - f(v, "jd")).abs() < 1e-8, "jd");
        let utc = at.to_offset(time::UtcOffset::UTC);
        let mjd = datetime_to_mjd(time::PrimitiveDateTime::new(utc.date(), utc.time()));
        assert!(
            (mjd - f(v, "mjd")).abs() < 1e-8,
            "mjd {mjd} vs {}",
            f(v, "mjd")
        );
    }
    for v in vectors()["jyear"].as_array().unwrap() {
        let Epoch::OfDate(year) = julian_epoch_of(instant(v, "at")) else {
            panic!("expected OfDate")
        };
        // AstroPy's jyear is TT-based; TT−UTC ≈ 69 s ≈ 2.2e-6 yr.
        assert!(
            (year - f(v, "epoch")).abs() < 1e-4,
            "epoch {year} vs {}",
            f(v, "epoch")
        );
    }
}

#[test]
fn sexagesimal_parsing_matches_astropy() {
    let vs = vectors();
    for v in vs["sexagesimal_ra"].as_array().unwrap() {
        let ours = parse_ra(v["s"].as_str().unwrap(), ParseMode::Strict)
            .unwrap()
            .degrees();
        assert!((ours - f(v, "deg")).abs() < 1e-9, "ra {ours}");
    }
    for v in vs["sexagesimal_dec"].as_array().unwrap() {
        let ours = parse_dec(v["s"].as_str().unwrap(), ParseMode::Strict)
            .unwrap()
            .degrees();
        assert!((ours - f(v, "deg")).abs() < 1e-9, "dec {ours}");
    }
}

#[test]
fn alt_az_matches_astropy_within_a_minute() {
    for v in vectors()["alt_az"].as_array().unwrap() {
        let h = alt_az(radec(&v["eq"]), instant(v, "at"), &site(&v["site"]));
        let drift = horizontal_sep_arcsec(
            h.altitude.degrees(),
            h.azimuth.degrees(),
            f(v, "alt_deg"),
            f(v, "az_deg"),
        );
        assert!(
            drift < 60.0,
            "alt-az drift {drift}″ (ours {}/{} vs {}/{})",
            h.altitude.degrees(),
            h.azimuth.degrees(),
            f(v, "alt_deg"),
            f(v, "az_deg")
        );
    }
}

#[test]
fn hour_angle_matches_astropy() {
    for v in vectors()["hour_angle"].as_array().unwrap() {
        let ha = hour_angle(radec(&v["eq"]), instant(v, "at"), &site(&v["site"])).degrees();
        assert!(
            wrap_diff_deg(ha, f(v, "ha_deg")) < 0.02,
            "ha {ha} vs {}",
            f(v, "ha_deg")
        );
    }
}

#[test]
fn parallactic_angle_matches_astroplan() {
    for v in vectors()["parallactic"].as_array().unwrap() {
        let q = parallactic_angle(radec(&v["eq"]), instant(v, "at"), &site(&v["site"])).degrees();
        assert!(
            wrap_diff_deg(q, f(v, "q_deg")) < 0.2,
            "q {q} vs {}",
            f(v, "q_deg")
        );
    }
}

#[test]
fn transit_matches_astroplan_within_a_minute() {
    for v in vectors()["transit"].as_array().unwrap() {
        let t = transit(radec(&v["eq"]), instant(v, "near"), &site(&v["site"]));
        let delta = (t - instant(v, "utc")).abs();
        assert!(
            delta < time::Duration::seconds(60),
            "transit off by {delta}"
        );
    }
}

#[test]
fn sun_position_matches_get_sun_within_a_minute() {
    // AstroPy's `get_sun` reports GCRS (J2000-aligned axes); ours is equinox
    // of date — precess back before comparing.
    for v in vectors()["sun"].as_array().unwrap() {
        let ours = precess(sun_position(instant(v, "at")), Epoch::J2000);
        let expected = Equatorial::j2000(
            Angle::from_degrees(f(v, "ra_deg")),
            Angle::from_degrees(f(v, "dec_deg")),
        )
        .unwrap();
        let drift = separation(ours, expected).arcseconds();
        assert!(drift < 60.0, "sun drift {drift}″ at {}", v["at"]);
    }
}

#[test]
fn moon_position_matches_get_body_within_two_minutes() {
    for v in vectors()["moon_geocentric"].as_array().unwrap() {
        let at = instant(v, "at");
        let ours = precess(moon_position(at), Epoch::J2000);
        let expected = Equatorial::j2000(
            Angle::from_degrees(f(v, "ra_deg")),
            Angle::from_degrees(f(v, "dec_deg")),
        )
        .unwrap();
        let drift = separation(ours, expected).arcminutes();
        assert!(drift < 2.0, "moon drift {drift}′ at {}", v["at"]);
        let dist = moon_distance_km(at);
        assert!(
            (dist - f(v, "distance_km")).abs() < 150.0,
            "Δ {dist} vs {}",
            f(v, "distance_km")
        );
    }
    for v in vectors()["moon_topocentric"].as_array().unwrap() {
        let ours = precess(
            moon_position_topocentric(instant(v, "at"), &site(&v["site"])),
            Epoch::J2000,
        );
        let expected = Equatorial::j2000(
            Angle::from_degrees(f(v, "ra_deg")),
            Angle::from_degrees(f(v, "dec_deg")),
        )
        .unwrap();
        let drift = separation(ours, expected).arcminutes();
        assert!(
            drift < 2.0,
            "topocentric moon drift {drift}′ at {}",
            v["at"]
        );
    }
}

#[test]
fn moon_illumination_matches_astroplan() {
    for v in vectors()["moon_illumination"].as_array().unwrap() {
        let k = moon_illumination(instant(v, "at"));
        assert!(
            (k - f(v, "k")).abs() < 0.01,
            "k {k} vs {} at {}",
            f(v, "k"),
            v["at"]
        );
    }
}

#[test]
fn twilight_matches_astroplan_within_a_minute() {
    for v in vectors()["twilight"].as_array().unwrap() {
        let outcome = twilight(
            Twilight::Astronomical,
            instant(v, "night"),
            &site(&v["site"]),
        );
        let TwilightOutcome::Night { dusk, dawn } = outcome else {
            panic!("expected Night, got {outcome:?}");
        };
        let dusk_delta = (dusk - instant(v, "dusk")).abs();
        let dawn_delta = (dawn - instant(v, "dawn")).abs();
        assert!(
            dusk_delta < time::Duration::seconds(60),
            "dusk off by {dusk_delta}"
        );
        assert!(
            dawn_delta < time::Duration::seconds(60),
            "dawn off by {dawn_delta}"
        );
    }
}

#[test]
fn moon_crossings_match_astroplan_within_three_minutes() {
    for v in vectors()["moon_crossings"].as_array().unwrap() {
        let site = site(&v["site"]);
        // Anchor inside the astroplan window so both solvers pick the same
        // lunar pass.
        let anchor = instant(v, "rise") + time::Duration::hours(3);
        let outcome = moon_crossings(Angle::from_degrees(0.0), anchor, &site);
        let CrossingOutcome::Crosses { rise, set } = outcome else {
            panic!("expected Crosses, got {outcome:?}");
        };
        let rise_delta = (rise - instant(v, "rise")).abs();
        let set_delta = (set - instant(v, "set")).abs();
        assert!(
            rise_delta < time::Duration::minutes(3),
            "rise off by {rise_delta}"
        );
        assert!(
            set_delta < time::Duration::minutes(3),
            "set off by {set_delta}"
        );
    }
}

#[test]
fn altitude_crossings_match_astroplan() {
    for v in vectors()["crossings"].as_array().unwrap() {
        let outcome = altitude_crossings(
            radec(&v["eq"]),
            Angle::from_degrees(f(v, "threshold_deg")),
            instant(v, "night"),
            &site(&v["site"]),
        );
        let CrossingOutcome::Crosses { rise, set } = outcome else {
            panic!("expected Crosses, got {outcome:?}");
        };
        let rise_delta = (rise - instant(v, "rise")).abs();
        let set_delta = (set - instant(v, "set")).abs();
        assert!(
            rise_delta < time::Duration::seconds(90),
            "rise off by {rise_delta}"
        );
        assert!(
            set_delta < time::Duration::seconds(90),
            "set off by {set_delta}"
        );
    }
}
