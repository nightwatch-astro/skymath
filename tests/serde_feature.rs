// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Exercises the optional `serde` feature end-to-end (FR-X3).
//! Only compiled/run with `--features serde`.
#![cfg(feature = "serde")]

use skymath::{
    Angle, Constellation, CrossingOutcome, Ecliptic, Epoch, Equatorial, Galactic, Horizontal,
    Location, Twilight, TwilightOutcome,
};
use time::macros::datetime;

fn round_trip<T>(value: &T) -> T
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    serde_json::from_str(&serde_json::to_string(value).unwrap()).unwrap()
}

#[test]
fn public_types_round_trip_through_json() {
    let pos = Equatorial::at_epoch(
        Angle::from_degrees(10.6847),
        Angle::from_degrees(41.2688),
        Epoch::OfDate(2026.5),
    )
    .unwrap();
    assert_eq!(pos, round_trip(&pos));

    let site = Location::new(Angle::from_degrees(52.155), Angle::from_degrees(4.485), 6.0).unwrap();
    assert_eq!(site, round_trip(&site));

    let horizontal = Horizontal {
        altitude: Angle::from_degrees(48.5),
        azimuth: Angle::from_degrees(307.4),
    };
    assert_eq!(horizontal, round_trip(&horizontal));

    let crossing = CrossingOutcome::Crosses {
        rise: datetime!(2026-07-11 21:12:03 UTC),
        set: datetime!(2026-07-12 03:41:59 UTC),
    };
    assert_eq!(crossing, round_trip(&crossing));
    assert_eq!(
        CrossingOutcome::AlwaysAbove,
        round_trip(&CrossingOutcome::AlwaysAbove)
    );

    let galactic = Galactic {
        l: Angle::from_degrees(121.17),
        b: Angle::from_degrees(-21.57),
    };
    assert_eq!(galactic, round_trip(&galactic));

    let ecliptic = Ecliptic {
        lambda: Angle::from_degrees(113.22),
        beta: Angle::from_degrees(6.68),
    };
    assert_eq!(ecliptic, round_trip(&ecliptic));

    assert_eq!(Twilight::Astronomical, round_trip(&Twilight::Astronomical));
    let night = TwilightOutcome::Night {
        dusk: datetime!(2026-10-15 18:55:00 UTC),
        dawn: datetime!(2026-10-16 05:10:00 UTC),
    };
    assert_eq!(night, round_trip(&night));
    assert_eq!(
        TwilightOutcome::NeverDark,
        round_trip(&TwilightOutcome::NeverDark)
    );
}

#[test]
fn constellation_wire_form_is_the_iau_abbreviation() {
    for c in Constellation::ALL {
        assert_eq!(c, round_trip(&c));
        assert_eq!(
            serde_json::to_string(&c).unwrap(),
            format!("\"{}\"", c.abbreviation())
        );
    }
}
