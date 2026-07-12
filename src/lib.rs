//! Planning-grade astronomy math for astrophotography tooling.
//!
//! Angles, equatorial coordinates with sexagesimal parsing and formatting,
//! great-circle geometry, precession, galactic and ecliptic frames, MJD/JD
//! and FITS date conversions, sidereal time, and observer-local quantities
//! (alt-azimuth, airmass, refraction, parallactic angle, transit times).
//! Precision is planning-grade (≈1 arcminute) by design — suitable for
//! framing, scheduling, and session planning, not telescope pointing or
//! astrometry. Apparent-place corrections (nutation, aberration, proper
//! motion) are out of scope.

pub mod angle;
pub mod coords;
pub mod error;
pub mod time;

pub use angle::{
    format_dec, format_ra, parse_dec, parse_ra, Angle, ParseMode, Separator, SexaStyle,
    ARCSEC_PER_RADIAN,
};
pub use coords::{
    apply_offset, position_angle, precess, separation, tangent_offset, Epoch, Equatorial,
    TangentOffset,
};
pub use error::{Error, Result};
pub use time::{
    datetime_to_mjd, format_date_obs, gmst, jd_to_mjd, julian_date, julian_epoch_of, lst,
    mjd_to_datetime, mjd_to_jd, parse_date_obs,
};
