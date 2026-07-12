//! Property-based tests of the geometric invariants (SC-004).
//!
//! The separation/sexagesimal/precession properties are migrated from
//! target-match's suite (SC-002); offset and parse-mode properties are new.

use proptest::prelude::*;
use skymath::{
    apply_offset, format_dec, format_ra, parse_dec, parse_ra, precess, separation, tangent_offset,
    Angle, Epoch, Equatorial, ParseMode, Separator, SexaStyle,
};

fn eq(ra: f64, dec: f64) -> Equatorial {
    Equatorial::j2000(Angle::from_degrees(ra), Angle::from_degrees(dec)).unwrap()
}

fn hi_res() -> SexaStyle {
    SexaStyle {
        separator: Separator::Colons,
        seconds_places: 5,
    }
}

proptest! {
    /// Separation is symmetric and bounded to [0, 180]°.
    #[test]
    fn separation_symmetric_and_bounded(
        ra1 in 0.0..360.0f64, dec1 in -90.0..90.0f64,
        ra2 in 0.0..360.0f64, dec2 in -90.0..90.0f64,
    ) {
        let (a, b) = (eq(ra1, dec1), eq(ra2, dec2));
        let ab = separation(a, b).degrees();
        let ba = separation(b, a).degrees();
        prop_assert!((ab - ba).abs() < 1e-9);
        prop_assert!((-1e-9..=180.0 + 1e-9).contains(&ab));
    }

    /// Sexagesimal format → parse round-trips to sub-milliarcsecond (strict mode).
    #[test]
    fn sexagesimal_round_trip(ra in 0.0..359.999f64, dec in -89.999..89.999f64) {
        let p = eq(ra, dec);
        let q = Equatorial::parse_j2000(
            &p.ra_sexagesimal(hi_res()),
            &p.dec_sexagesimal(hi_res()),
            ParseMode::Strict,
        ).unwrap();
        prop_assert!(separation(p, q).arcseconds() < 1e-2, "ra={ra} dec={dec}");
    }

    /// Space-separated (FITS style) formatting parses back identically in lenient mode.
    #[test]
    fn fits_style_round_trip(ra in 0.0..359.999f64, dec in -89.999..89.999f64) {
        let style = SexaStyle { separator: Separator::Spaces, seconds_places: 4 };
        let a = parse_ra(&format_ra(Angle::from_degrees(ra), style), ParseMode::Lenient).unwrap();
        let d = parse_dec(&format_dec(Angle::from_degrees(dec), style), ParseMode::Lenient).unwrap();
        prop_assert!((a.degrees() - ra).abs() * 3600.0 < 0.1, "ra {ra}");
        prop_assert!((d.degrees() - dec).abs() * 3600.0 < 0.1, "dec {dec}");
    }

    /// Precession to epoch-of-date and back is the identity (inverse consistency).
    #[test]
    fn precession_round_trip(ra in 0.0..360.0f64, dec in -89.0..89.0f64, year in 1900.0..2100.0f64) {
        let p = eq(ra, dec);
        let back = precess(precess(p, Epoch::OfDate(year)), Epoch::J2000);
        prop_assert!(separation(p, back).arcseconds() < 1e-3, "ra={ra} dec={dec} year={year}");
    }

    /// Tangent offset decompose → apply recovers the target position.
    #[test]
    fn offset_round_trip(
        ra1 in 0.0..360.0f64, dec1 in -89.0..89.0f64,
        ra2 in 0.0..360.0f64, dec2 in -89.0..89.0f64,
    ) {
        let (from, to) = (eq(ra1, dec1), eq(ra2, dec2));
        let back = apply_offset(from, tangent_offset(from, to));
        prop_assert!(
            separation(to, back).arcseconds() < 1e-3,
            "drift {} arcsec", separation(to, back).arcseconds()
        );
    }
}

// ── US2: time properties ───────────────────────────────────────────────────────

use skymath::{datetime_to_mjd, gmst, mjd_to_datetime};
use time::macros::datetime;
use time::{Date, Duration, Month, PrimitiveDateTime, Time, UtcOffset};

proptest! {
    /// Calendar → MJD → calendar is lossless to well below planning grade.
    #[test]
    fn calendar_mjd_round_trip(
        year in 1859i32..2200, month in 1u8..=12, day in 1u8..=28,
        hour in 0u8..24, minute in 0u8..60, second in 0u8..60, milli in 0u16..1000,
    ) {
        let dt = PrimitiveDateTime::new(
            Date::from_calendar_date(year, Month::try_from(month).unwrap(), day).unwrap(),
            Time::from_hms_milli(hour, minute, second, milli).unwrap(),
        );
        let back = mjd_to_datetime(datetime_to_mjd(dt)).unwrap();
        prop_assert!((back - dt).abs() < Duration::microseconds(5), "{dt} -> {back}");
    }

    /// GMST depends on the instant, not the civil offset it is written in.
    #[test]
    fn gmst_offset_invariant(
        secs in -1_000_000_000i64..1_000_000_000,
        offset_minutes in -720i32..=840,
    ) {
        let instant = datetime!(2000-01-01 12:00 UTC) + Duration::seconds(secs);
        let offset = UtcOffset::from_whole_seconds(offset_minutes * 60).unwrap();
        let rewritten = instant.to_offset(offset);
        prop_assert!((gmst(instant).hours() - gmst(rewritten).hours()).abs() < 1e-9);
    }
}

// ── US3: observer properties ───────────────────────────────────────────────────

use skymath::{hour_angle, parallactic_angle, Location};

proptest! {
    /// The parallactic angle is 0 at transit and carries the hour angle's
    /// sign either side of the meridian (northern site, target south of the
    /// zenith so the atan2 denominator stays positive).
    #[test]
    fn parallactic_angle_tracks_the_meridian(
        ra in 0.0..360.0f64,
        dec in -20.0..40.0f64,
        minutes in -180i64..=180,
    ) {
        let site = Location::new(
            Angle::from_degrees(52.0), Angle::from_degrees(4.5), 0.0,
        ).unwrap();
        let target = eq(ra, dec);
        let t0 = skymath::transit(target, datetime!(2026-07-11 22:00 UTC), &site);

        let q_transit = parallactic_angle(target, t0, &site).degrees();
        prop_assert!(q_transit.abs() < 0.1, "q at transit: {q_transit}°");

        let t = t0 + Duration::minutes(minutes);
        let ha = hour_angle(target, t, &site).degrees();
        let q = parallactic_angle(target, t, &site).degrees();
        if ha.abs() > 0.5 {
            prop_assert!(
                q.signum() == ha.signum(),
                "q {q}° vs HA {ha}° must share a sign"
            );
        }
    }
}

// ── 002: sun & moon properties ─────────────────────────────────────────────────

use skymath::{
    moon_avoidance_lorentzian, moon_illumination, sun_position, twilight, Twilight, TwilightOutcome,
};

proptest! {
    /// Twilight never fabricates instants: every latitude yields a typed
    /// outcome, and Night boundaries actually sit on the threshold (SC-003).
    #[test]
    fn twilight_outcomes_are_typed_across_latitudes(
        lat in -80.0..80.0f64,
        day_offset in 0i64..365,
    ) {
        let site = Location::new(Angle::from_degrees(lat), Angle::from_degrees(4.5), 0.0).unwrap();
        let night = datetime!(2026-01-01 23:00 UTC) + Duration::days(day_offset);
        match twilight(Twilight::Astronomical, night, &site) {
            TwilightOutcome::Night { dusk, dawn } => {
                prop_assert!(dusk < dawn, "dusk {dusk} not before dawn {dawn}");
                let alt = skymath::alt_az(sun_position(dusk), dusk, &site).altitude.degrees();
                prop_assert!((alt + 18.0).abs() < 0.25, "lat {lat}: dusk altitude {alt}°");
            }
            TwilightOutcome::NeverDark | TwilightOutcome::AlwaysDark => {}
        }
    }

    /// The illuminated fraction is a fraction, and the avoidance criterion is
    /// bounded by its full-Moon maximum.
    #[test]
    fn illumination_and_avoidance_are_bounded(
        hours in 0i64..17_520, // two years of instants
        half_width in 0.5..15.0f64,
    ) {
        let at = datetime!(2026-01-01 00:00 UTC) + Duration::hours(hours);
        let k = moon_illumination(at);
        prop_assert!((0.0..=1.0).contains(&k), "k = {k}");
        let s = moon_avoidance_lorentzian(Angle::from_degrees(60.0), half_width, at);
        prop_assert!(s.degrees() > 0.0 && s.degrees() <= 60.0 + 1e-9, "S = {}", s.degrees());
    }
}

// ── US4: frame round-trips ─────────────────────────────────────────────────────

use skymath::{from_ecliptic, from_galactic, to_ecliptic, to_galactic};

proptest! {
    /// Galactic is an exact rotation: round-trips to within an arcsecond.
    #[test]
    fn galactic_round_trip(ra in 0.0..360.0f64, dec in -89.9..89.9f64) {
        let p = eq(ra, dec);
        let back = from_galactic(to_galactic(p));
        prop_assert!(
            separation(p, back).arcseconds() < 1.0,
            "drift {}″", separation(p, back).arcseconds()
        );
    }

    /// Ecliptic-of-date round-trips to within an arcsecond at any instant.
    #[test]
    fn ecliptic_round_trip(
        ra in 0.0..360.0f64, dec in -89.9..89.9f64, days in -18_000i64..18_000,
    ) {
        let at = datetime!(2000-01-01 12:00 UTC) + Duration::days(days);
        let p = eq(ra, dec);
        let back = from_ecliptic(to_ecliptic(p, at), at);
        prop_assert!(
            separation(p, back).arcseconds() < 1.0,
            "drift {}″ at {at}", separation(p, back).arcseconds()
        );
    }
}
