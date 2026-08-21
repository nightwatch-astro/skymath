// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Exhaustive check that formatted sexagesimal output stays inside the
//! sexagesimal domain: neither the minutes nor the seconds field may reach 60.
//!
//! Hand-picked cases do not cover this: the defect it guards against was a
//! float-error tail, so the sweep walks the whole display domain at
//! millidegree resolution.

use skymath::{format_dec, format_ra, Angle, Separator, SexaStyle};

fn style(seconds_places: u8) -> SexaStyle {
    SexaStyle {
        separator: Separator::Colons,
        seconds_places,
    }
}

/// Minutes and seconds fields of `hh:mm:ss[.sss]`, sign stripped.
fn minute_and_second_fields(formatted: &str) -> (f64, f64) {
    let mut fields = formatted
        .trim_start_matches(['+', '-'])
        .split(':')
        .skip(1)
        .map(|f| f.parse::<f64>().expect("numeric sexagesimal field"));
    let minutes = fields.next().expect("minutes field");
    let seconds = fields.next().expect("seconds field");
    (minutes, seconds)
}

fn out_of_domain(formatted: &str) -> bool {
    let (minutes, seconds) = minute_and_second_fields(formatted);
    minutes >= 60.0 || seconds >= 60.0
}

#[test]
fn rounding_carry_reaches_the_minute_field() {
    let integer_seconds = style(0);
    assert_eq!(
        format_ra(Angle::from_degrees(15.25), integer_seconds),
        "01:01:00"
    );
    assert_eq!(
        format_ra(Angle::from_degrees(16.0), integer_seconds),
        "01:04:00"
    );
    assert_eq!(
        format_dec(Angle::from_degrees(-89.85), integer_seconds),
        "-89:51:00"
    );
    assert_eq!(
        format_ra(Angle::from_degrees(63.75), style(2)),
        "04:15:00.00"
    );
}

#[test]
fn millidegree_sweep_never_emits_a_60_field() {
    for places in [0u8, 2, 5] {
        let style = style(places);

        let mut ra_offenders = Vec::new();
        for millidegrees in 0..360_000 {
            let degrees = f64::from(millidegrees) / 1000.0;
            let formatted = format_ra(Angle::from_degrees(degrees), style);
            if out_of_domain(&formatted) {
                ra_offenders.push((degrees, formatted));
            }
        }

        let mut dec_offenders = Vec::new();
        for millidegrees in -90_000..=90_000 {
            let degrees = f64::from(millidegrees) / 1000.0;
            let formatted = format_dec(Angle::from_degrees(degrees), style);
            if out_of_domain(&formatted) {
                dec_offenders.push((degrees, formatted));
            }
        }

        assert!(
            ra_offenders.is_empty() && dec_offenders.is_empty(),
            "seconds_places {places}: {} RA and {} Dec values formatted outside the \
             sexagesimal domain; first RA {:?}, first Dec {:?}",
            ra_offenders.len(),
            dec_offenders.len(),
            ra_offenders.first(),
            dec_offenders.first(),
        );
    }
}
