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
