#![doc = include_str!("../README.md")]
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
#![warn(missing_docs)]

pub mod angle;
pub mod constellation;
mod constellation_data;
pub mod coords;
pub mod error;
pub mod frames;
#[doc = include_str!("../docs/guide.md")]
pub mod guide {}
pub mod moon;
pub mod observer;
pub mod sun;
pub mod time;

// `#[doc(inline)]` puts each item's canonical rustdoc page at the crate root
// (`skymath/struct.Angle.html`, `skymath/fn.alt_az.html`), matching the
// re-exported-from-the-root API and the docs.rs links in the README and guide;
// without it the canonical page lives at the submodule path instead.
#[doc(inline)]
pub use angle::{
    circular_distance, circular_mean, format_dec, format_ra, parse_dec, parse_ra, Angle,
    CircularMean, ParseMode, Separator, SexaStyle, ARCSEC_PER_RADIAN,
};
#[doc(inline)]
pub use constellation::{constellation, Constellation};
#[doc(inline)]
pub use coords::{
    apply_offset, gnomonic_project, gnomonic_unproject, position_angle, precess, separation,
    tangent_offset, transport_position_angle, Epoch, Equatorial, GnomonicPoint, TangentOffset,
};
#[doc(inline)]
pub use error::{Error, Result};
#[doc(inline)]
pub use frames::{from_ecliptic, from_galactic, to_ecliptic, to_galactic, Ecliptic, Galactic};
#[doc(inline)]
pub use moon::{
    lunar_separation, moon_avoidance_lorentzian, moon_crossings, moon_distance_km,
    moon_illumination, moon_phase_angle, moon_position, moon_position_topocentric,
};
#[doc(inline)]
pub use observer::{
    airmass, alt_az, altitude_crossings, hour_angle, parallactic_angle,
    refraction_apparent_to_true, refraction_true_to_apparent, transit, CrossingOutcome, Horizontal,
    Location,
};
#[doc(inline)]
pub use sun::{sun_position, twilight, Twilight, TwilightOutcome};
#[doc(inline)]
pub use time::{
    datetime_to_mjd, format_date_obs, gmst, jd_to_mjd, julian_date, julian_epoch_of, lst,
    mjd_to_datetime, mjd_to_jd, parse_date_obs,
};
