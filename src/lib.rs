//! Planning-grade astronomy math for astrophotography tooling.
//!
//! Angles, equatorial coordinates with sexagesimal parsing and formatting,
//! great-circle geometry, precession, galactic and ecliptic frames, MJD/JD
//! and FITS date conversions, sidereal time, observer-local quantities
//! (alt-azimuth, airmass, refraction, parallactic angle, transit times),
//! Sun/Moon ephemerides, and IAU constellation identification.
//! Precision is planning-grade (≈1 arcminute) by design — suitable for
//! framing, scheduling, and session planning, not telescope pointing or
//! astrometry. Apparent-place corrections (nutation, aberration, proper
//! motion) are out of scope.
//!
//! Everything is re-exported from the crate root; instants use the [`time`]
//! crate's types, and functions taking an `OffsetDateTime` fold the offset in
//! internally, so local civil time cannot skew results.
//!
//! ```
//! use skymath::{separation, Equatorial, ParseMode};
//!
//! let m31 = Equatorial::parse_j2000("00:42:44.3", "+41:16:09", ParseMode::Strict)?;
//! let m110 = Equatorial::parse_j2000("00:40:22.1", "+41:41:07", ParseMode::Lenient)?;
//! let sep = separation(m31, m110);
//! assert!((sep.arcminutes() - 36.5).abs() < 1.0);
//! # Ok::<(), skymath::Error>(())
//! ```
//!
//! Enable the `serde` feature for `Serialize`/`Deserialize` derives on all
//! public types.
#![warn(missing_docs)]

pub mod angle;
pub mod constellation;
mod constellation_data;
pub mod coords;
pub mod error;
pub mod frames;
pub mod moon;
pub mod observer;
pub mod sun;
pub mod time;

pub use angle::{
    format_dec, format_ra, parse_dec, parse_ra, Angle, ParseMode, Separator, SexaStyle,
    ARCSEC_PER_RADIAN,
};
pub use constellation::{constellation, Constellation};
pub use coords::{
    apply_offset, position_angle, precess, separation, tangent_offset, Epoch, Equatorial,
    TangentOffset,
};
pub use error::{Error, Result};
pub use frames::{from_ecliptic, from_galactic, to_ecliptic, to_galactic, Ecliptic, Galactic};
pub use moon::{
    lunar_separation, moon_avoidance_lorentzian, moon_crossings, moon_distance_km,
    moon_illumination, moon_phase_angle, moon_position, moon_position_topocentric,
};
pub use observer::{
    airmass, alt_az, altitude_crossings, hour_angle, parallactic_angle,
    refraction_apparent_to_true, refraction_true_to_apparent, transit, CrossingOutcome, Horizontal,
    Location,
};
pub use sun::{sun_position, twilight, Twilight, TwilightOutcome};
pub use time::{
    datetime_to_mjd, format_date_obs, gmst, jd_to_mjd, julian_date, julian_epoch_of, lst,
    mjd_to_datetime, mjd_to_jd, parse_date_obs,
};
