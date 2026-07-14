// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Vectors exercising the alt-azimuth transform ported from `gaker/astro-math`
//! 0.2.1 (`src/tests/transforms.rs`, see NOTICE) — SC-003.
//!
//! Each block cites the donor test it was lifted from. Provenance correction:
//! the donor's Kitt Peak numbers claimed AstroPy verification but were in fact
//! its own unprecessed geometric output (AstroPy disagrees by ~10′ because the
//! donor compared J2000 RA against of-date sidereal time). The expected values
//! below are genuine AstroPy 8.0.1 outputs (pressure=0), regenerable with
//! `scripts/gen_astropy_vectors.py`; skymath matches them because its observer
//! functions precess to the epoch of date internally.

use skymath::{alt_az, Angle, Equatorial, Location};
use time::macros::datetime;

fn eq(ra: f64, dec: f64) -> Equatorial {
    Equatorial::j2000(Angle::from_degrees(ra), Angle::from_degrees(dec)).unwrap()
}

fn site(lat: f64, lon: f64, elev: f64) -> Location {
    Location::new(Angle::from_degrees(lat), Angle::from_degrees(lon), elev).unwrap()
}

#[test]
fn vega_from_kitt_peak_matches_astropy() {
    // Case from astro-math `test_ra_dec_to_alt_az_astropy_crosscheck`: Vega
    // (α Lyr) from Kitt Peak, 2024-08-04T06:00 UTC. Expected values are
    // AstroPy 8.0.1 (AltAz, pressure=0): alt 77.906675°, az 307.877201°.
    // Tolerance 1′ on altitude (the SC-003 public claim; AstroPy's apparent
    // place adds nutation ~17″ + aberration ~20″ we don't model) and the
    // zenith-amplified equivalent on azimuth (1′ / cos 78° ≈ 0.08°).
    let observer = site(31.9583, -111.6, 2120.0);
    let vega = eq(279.234_734_79, 38.783_688_96);
    let h = alt_az(vega, datetime!(2024-08-04 06:00 UTC), &observer);
    assert!(
        (h.altitude.degrees() - 77.906_675).abs() < 1.0 / 60.0,
        "alt {}",
        h.altitude.degrees()
    );
    assert!(
        (h.azimuth.degrees() - 307.877_201).abs() < 0.08,
        "az {}",
        h.azimuth.degrees()
    );
}

#[test]
fn azimuth_stays_normalized_west_of_meridian() {
    // astro-math `test_ra_dec_to_alt_az_negative_azimuth_wrap`.
    let h = alt_az(
        eq(180.0, -10.0),
        datetime!(2024-01-01 12:00 UTC),
        &site(0.0, 0.0, 0.0),
    );
    let az = h.azimuth.degrees();
    assert!((0.0..360.0).contains(&az), "az {az}");
}

#[test]
fn zenith_azimuth_is_a_meridian_side() {
    // astro-math `test_ra_dec_to_alt_az_zenith_edge_case`: a target at the
    // observer's latitude transiting through the zenith has a degenerate
    // azimuth; the convention reports 0° (east side) or 180° (west side).
    let observer = site(45.0, 0.0, 0.0);
    let target = eq(0.0, 45.0);
    let h = alt_az(target, datetime!(2024-03-20 12:00 UTC), &observer);
    if h.altitude.degrees() > 89.9 {
        let az = h.azimuth.degrees();
        assert!(az == 0.0 || az == 180.0, "zenith az {az}");
    }
}

#[test]
fn polar_observer_stays_in_range() {
    // astro-math `test_ra_dec_to_alt_az_polar_observer`: Polaris-like target
    // from 89.9° N must produce in-range values, no NaN, no panic.
    let h = alt_az(
        eq(37.954_56, 89.264_11),
        datetime!(2024-06-21 00:00 UTC),
        &site(89.9, 0.0, 0.0),
    );
    assert!((-90.0..=90.0).contains(&h.altitude.degrees()));
    assert!((0.0..360.0).contains(&h.azimuth.degrees()));
}

#[test]
fn horizon_target_survives_acos_domain() {
    // astro-math `test_ra_dec_to_alt_az_numerical_stability`: an
    // equator-observer horizon geometry that can push cos(az) past ±1.
    let h = alt_az(
        eq(90.0, 0.0),
        datetime!(2024-03-20 06:00 UTC),
        &site(0.0, 0.0, 0.0),
    );
    assert!((-90.0..=90.0).contains(&h.altitude.degrees()));
    assert!((0.0..360.0).contains(&h.azimuth.degrees()));
}
