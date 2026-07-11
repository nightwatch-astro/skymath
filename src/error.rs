//! Error type for `skymath`.
//!
//! Every fallible entry point returns [`Result`]. Errors arise only at
//! construction and parse boundaries and at documented formula domain edges —
//! computation on already-validated types is infallible.

use thiserror::Error;

/// Everything that can go wrong constructing or parsing `skymath` values.
#[derive(Debug, Clone, PartialEq, Error)]
#[non_exhaustive]
pub enum Error {
    /// A coordinate or location string could not be parsed (empty, corrupt
    /// token, malformed sexagesimal, or negative/overflowing minutes/seconds).
    #[error("could not parse coordinate: {0}")]
    ParseCoord(String),

    /// A date/time string could not be parsed as a FITS-style datetime.
    #[error("could not parse date: {0}")]
    ParseDate(String),

    /// A numeric quantity was outside its valid domain (RA `[0, 360)`°, Dec
    /// `[-90, 90]`°, latitude/longitude bounds, non-finite input, or a formula
    /// validity edge such as airmass below the horizon).
    #[error("{what} out of range: {value}")]
    OutOfRange {
        /// Which quantity was out of range (e.g. `"declination"`).
        what: &'static str,
        /// The offending value.
        value: f64,
    },
}

/// Convenience alias for `Result<T, `[`Error`](enum@Error)`>`.
pub type Result<T> = core::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_messages() {
        assert_eq!(
            Error::ParseCoord("x".into()).to_string(),
            "could not parse coordinate: x"
        );
        assert_eq!(
            Error::ParseDate("y".into()).to_string(),
            "could not parse date: y"
        );
        assert_eq!(
            Error::OutOfRange {
                what: "declination",
                value: 91.0
            }
            .to_string(),
            "declination out of range: 91"
        );
    }
}
