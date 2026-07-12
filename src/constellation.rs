//! Constellation identification from the IAU constellation boundaries.
//!
//! Provenance: identification walks the Roman (1987, PASP 99, 695)
//! precomputed boundary table (ADC/CDS catalogue VI/42 — the Delporte 1930
//! IAU boundaries as declination-zone records at epoch B1875.0), generated
//! into `constellation_data.rs` by `scripts/gen_constellation_table.py`.
//! Inputs are precessed to B1875.0 with the crate's IAU-1976 model — the same
//! "FK5 stands in for the original FK4 frame" approximation AstroPy's
//! `get_constellation` makes; the difference is sub-arcsecond and can only
//! matter for coordinates within ~1″ of a boundary. Names follow the official
//! IAU list (including "Boötes", "Chamaeleon", "Ophiuchus",
//! "Piscis Austrinus").

use core::fmt;
use core::str::FromStr;

use crate::constellation_data::ZONES;
use crate::coords::{precess, Epoch, Equatorial};
use crate::error::Error;

/// B1875.0 (a Besselian epoch, JD 2405889.2585505) expressed as the Julian
/// year [`precess`] expects: `2000 + (JD − 2451545)/365.25`. A day-level slip
/// here moves coordinates ~0.07″ — far inside the boundary fine print above.
const B1875_JULIAN_YEAR: f64 = 1875.0013923;

/// One of the 88 IAU constellations.
///
/// Variants use the official IAU 3-letter abbreviation casing; the derived
/// `serde` form (behind the `serde` feature) is therefore exactly that
/// abbreviation. [`Constellation::name`] gives the full Latin name with IAU
/// spelling, which is also the [`fmt::Display`] form; parsing via [`FromStr`]
/// accepts the abbreviation case-insensitively.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Constellation {
    /// Andromeda (And).
    And,
    /// Antlia (Ant).
    Ant,
    /// Apus (Aps).
    Aps,
    /// Aquarius (Aqr).
    Aqr,
    /// Aquila (Aql).
    Aql,
    /// Ara (Ara).
    Ara,
    /// Aries (Ari).
    Ari,
    /// Auriga (Aur).
    Aur,
    /// Boötes (Boo).
    Boo,
    /// Caelum (Cae).
    Cae,
    /// Camelopardalis (Cam).
    Cam,
    /// Cancer (Cnc).
    Cnc,
    /// Canes Venatici (CVn).
    CVn,
    /// Canis Major (CMa).
    CMa,
    /// Canis Minor (CMi).
    CMi,
    /// Capricornus (Cap).
    Cap,
    /// Carina (Car).
    Car,
    /// Cassiopeia (Cas).
    Cas,
    /// Centaurus (Cen).
    Cen,
    /// Cepheus (Cep).
    Cep,
    /// Cetus (Cet).
    Cet,
    /// Chamaeleon (Cha).
    Cha,
    /// Circinus (Cir).
    Cir,
    /// Columba (Col).
    Col,
    /// Coma Berenices (Com).
    Com,
    /// Corona Australis (CrA).
    CrA,
    /// Corona Borealis (CrB).
    CrB,
    /// Corvus (Crv).
    Crv,
    /// Crater (Crt).
    Crt,
    /// Crux (Cru).
    Cru,
    /// Cygnus (Cyg).
    Cyg,
    /// Delphinus (Del).
    Del,
    /// Dorado (Dor).
    Dor,
    /// Draco (Dra).
    Dra,
    /// Equuleus (Equ).
    Equ,
    /// Eridanus (Eri).
    Eri,
    /// Fornax (For).
    For,
    /// Gemini (Gem).
    Gem,
    /// Grus (Gru).
    Gru,
    /// Hercules (Her).
    Her,
    /// Horologium (Hor).
    Hor,
    /// Hydra (Hya).
    Hya,
    /// Hydrus (Hyi).
    Hyi,
    /// Indus (Ind).
    Ind,
    /// Lacerta (Lac).
    Lac,
    /// Leo (Leo).
    Leo,
    /// Leo Minor (LMi).
    LMi,
    /// Lepus (Lep).
    Lep,
    /// Libra (Lib).
    Lib,
    /// Lupus (Lup).
    Lup,
    /// Lynx (Lyn).
    Lyn,
    /// Lyra (Lyr).
    Lyr,
    /// Mensa (Men).
    Men,
    /// Microscopium (Mic).
    Mic,
    /// Monoceros (Mon).
    Mon,
    /// Musca (Mus).
    Mus,
    /// Norma (Nor).
    Nor,
    /// Octans (Oct).
    Oct,
    /// Ophiuchus (Oph).
    Oph,
    /// Orion (Ori).
    Ori,
    /// Pavo (Pav).
    Pav,
    /// Pegasus (Peg).
    Peg,
    /// Perseus (Per).
    Per,
    /// Phoenix (Phe).
    Phe,
    /// Pictor (Pic).
    Pic,
    /// Pisces (Psc).
    Psc,
    /// Piscis Austrinus (PsA).
    PsA,
    /// Puppis (Pup).
    Pup,
    /// Pyxis (Pyx).
    Pyx,
    /// Reticulum (Ret).
    Ret,
    /// Sagitta (Sge).
    Sge,
    /// Sagittarius (Sgr).
    Sgr,
    /// Scorpius (Sco).
    Sco,
    /// Sculptor (Scl).
    Scl,
    /// Scutum (Sct).
    Sct,
    /// Serpens (Ser) — both its disjoint regions (Caput and Cauda).
    Ser,
    /// Sextans (Sex).
    Sex,
    /// Taurus (Tau).
    Tau,
    /// Telescopium (Tel).
    Tel,
    /// Triangulum (Tri).
    Tri,
    /// Triangulum Australe (TrA).
    TrA,
    /// Tucana (Tuc).
    Tuc,
    /// Ursa Major (UMa).
    UMa,
    /// Ursa Minor (UMi).
    UMi,
    /// Vela (Vel).
    Vel,
    /// Virgo (Vir).
    Vir,
    /// Volans (Vol).
    Vol,
    /// Vulpecula (Vul).
    Vul,
}

/// Official IAU 3-letter abbreviations, indexed by variant order.
const ABBREVIATIONS: [&str; 88] = [
    "And", "Ant", "Aps", "Aqr", "Aql", "Ara", "Ari", "Aur", "Boo", "Cae", "Cam", "Cnc", "CVn",
    "CMa", "CMi", "Cap", "Car", "Cas", "Cen", "Cep", "Cet", "Cha", "Cir", "Col", "Com", "CrA",
    "CrB", "Crv", "Crt", "Cru", "Cyg", "Del", "Dor", "Dra", "Equ", "Eri", "For", "Gem", "Gru",
    "Her", "Hor", "Hya", "Hyi", "Ind", "Lac", "Leo", "LMi", "Lep", "Lib", "Lup", "Lyn", "Lyr",
    "Men", "Mic", "Mon", "Mus", "Nor", "Oct", "Oph", "Ori", "Pav", "Peg", "Per", "Phe", "Pic",
    "Psc", "PsA", "Pup", "Pyx", "Ret", "Sge", "Sgr", "Sco", "Scl", "Sct", "Ser", "Sex", "Tau",
    "Tel", "Tri", "TrA", "Tuc", "UMa", "UMi", "Vel", "Vir", "Vol", "Vul",
];

/// Full Latin names (official IAU spellings), indexed by variant order.
const NAMES: [&str; 88] = [
    "Andromeda",
    "Antlia",
    "Apus",
    "Aquarius",
    "Aquila",
    "Ara",
    "Aries",
    "Auriga",
    "Boötes",
    "Caelum",
    "Camelopardalis",
    "Cancer",
    "Canes Venatici",
    "Canis Major",
    "Canis Minor",
    "Capricornus",
    "Carina",
    "Cassiopeia",
    "Centaurus",
    "Cepheus",
    "Cetus",
    "Chamaeleon",
    "Circinus",
    "Columba",
    "Coma Berenices",
    "Corona Australis",
    "Corona Borealis",
    "Corvus",
    "Crater",
    "Crux",
    "Cygnus",
    "Delphinus",
    "Dorado",
    "Draco",
    "Equuleus",
    "Eridanus",
    "Fornax",
    "Gemini",
    "Grus",
    "Hercules",
    "Horologium",
    "Hydra",
    "Hydrus",
    "Indus",
    "Lacerta",
    "Leo",
    "Leo Minor",
    "Lepus",
    "Libra",
    "Lupus",
    "Lynx",
    "Lyra",
    "Mensa",
    "Microscopium",
    "Monoceros",
    "Musca",
    "Norma",
    "Octans",
    "Ophiuchus",
    "Orion",
    "Pavo",
    "Pegasus",
    "Perseus",
    "Phoenix",
    "Pictor",
    "Pisces",
    "Piscis Austrinus",
    "Puppis",
    "Pyxis",
    "Reticulum",
    "Sagitta",
    "Sagittarius",
    "Scorpius",
    "Sculptor",
    "Scutum",
    "Serpens",
    "Sextans",
    "Taurus",
    "Telescopium",
    "Triangulum",
    "Triangulum Australe",
    "Tucana",
    "Ursa Major",
    "Ursa Minor",
    "Vela",
    "Virgo",
    "Volans",
    "Vulpecula",
];

impl Constellation {
    /// All 88 constellations, ordered alphabetically by abbreviation (the
    /// variant declaration order).
    pub const ALL: [Constellation; 88] = [
        Constellation::And,
        Constellation::Ant,
        Constellation::Aps,
        Constellation::Aqr,
        Constellation::Aql,
        Constellation::Ara,
        Constellation::Ari,
        Constellation::Aur,
        Constellation::Boo,
        Constellation::Cae,
        Constellation::Cam,
        Constellation::Cnc,
        Constellation::CVn,
        Constellation::CMa,
        Constellation::CMi,
        Constellation::Cap,
        Constellation::Car,
        Constellation::Cas,
        Constellation::Cen,
        Constellation::Cep,
        Constellation::Cet,
        Constellation::Cha,
        Constellation::Cir,
        Constellation::Col,
        Constellation::Com,
        Constellation::CrA,
        Constellation::CrB,
        Constellation::Crv,
        Constellation::Crt,
        Constellation::Cru,
        Constellation::Cyg,
        Constellation::Del,
        Constellation::Dor,
        Constellation::Dra,
        Constellation::Equ,
        Constellation::Eri,
        Constellation::For,
        Constellation::Gem,
        Constellation::Gru,
        Constellation::Her,
        Constellation::Hor,
        Constellation::Hya,
        Constellation::Hyi,
        Constellation::Ind,
        Constellation::Lac,
        Constellation::Leo,
        Constellation::LMi,
        Constellation::Lep,
        Constellation::Lib,
        Constellation::Lup,
        Constellation::Lyn,
        Constellation::Lyr,
        Constellation::Men,
        Constellation::Mic,
        Constellation::Mon,
        Constellation::Mus,
        Constellation::Nor,
        Constellation::Oct,
        Constellation::Oph,
        Constellation::Ori,
        Constellation::Pav,
        Constellation::Peg,
        Constellation::Per,
        Constellation::Phe,
        Constellation::Pic,
        Constellation::Psc,
        Constellation::PsA,
        Constellation::Pup,
        Constellation::Pyx,
        Constellation::Ret,
        Constellation::Sge,
        Constellation::Sgr,
        Constellation::Sco,
        Constellation::Scl,
        Constellation::Sct,
        Constellation::Ser,
        Constellation::Sex,
        Constellation::Tau,
        Constellation::Tel,
        Constellation::Tri,
        Constellation::TrA,
        Constellation::Tuc,
        Constellation::UMa,
        Constellation::UMi,
        Constellation::Vel,
        Constellation::Vir,
        Constellation::Vol,
        Constellation::Vul,
    ];

    /// The official IAU 3-letter abbreviation (e.g. `"UMi"`).
    #[must_use]
    pub fn abbreviation(self) -> &'static str {
        ABBREVIATIONS[self as usize]
    }

    /// The full Latin name with official IAU spelling (e.g. `"Ursa Minor"`,
    /// `"Boötes"`).
    #[must_use]
    pub fn name(self) -> &'static str {
        NAMES[self as usize]
    }
}

impl fmt::Display for Constellation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl FromStr for Constellation {
    type Err = Error;

    /// Parse an IAU 3-letter abbreviation, case-insensitively (`"UMi"`,
    /// `"umi"`, `"UMI"` → [`Constellation::UMi`]).
    ///
    /// # Errors
    /// [`Error::UnknownConstellation`] if the input is not one of the 88
    /// official abbreviations.
    fn from_str(s: &str) -> Result<Self, Error> {
        Constellation::ALL
            .into_iter()
            .find(|c| c.abbreviation().eq_ignore_ascii_case(s))
            .ok_or_else(|| Error::UnknownConstellation(s.to_string()))
    }
}

/// The constellation containing an equatorial coordinate.
///
/// Total over the celestial sphere: every valid [`Equatorial`] (J2000 or
/// epoch-of-date — the input's epoch is honoured) maps to exactly one of the
/// 88 IAU constellations. The coordinate is precessed to B1875.0 and located
/// in the Roman (1987) zone table; a point exactly on a boundary resolves
/// deterministically by the table's half-open convention (RA upper edges
/// exclusive, lower edges inclusive, first matching zone wins).
///
/// ```
/// use skymath::{constellation, Constellation, Equatorial, ParseMode};
///
/// let m31 = Equatorial::parse_j2000("00:42:44.3", "+41:16:09", ParseMode::Strict)?;
/// assert_eq!(constellation(m31), Constellation::And);
/// assert_eq!(constellation(m31).name(), "Andromeda");
/// # Ok::<(), skymath::Error>(())
/// ```
#[must_use]
pub fn constellation(coord: Equatorial) -> Constellation {
    let b1875 = precess(coord, Epoch::OfDate(B1875_JULIAN_YEAR));
    lookup_b1875(b1875.ra().hours(), b1875.dec().degrees())
}

/// Roman (1987) table walk in B1875 coordinates: the first record with
/// `dec >= dec_lo && ra_lo <= ra < ra_hi` wins. Total by construction — the
/// final record spans 0–24ʰ down to −90°.
fn lookup_b1875(ra_hours: f64, dec_degrees: f64) -> Constellation {
    let ra = ra_hours.rem_euclid(24.0);
    for &(ra_lo, ra_hi, dec_lo, c) in ZONES.iter() {
        if dec_degrees >= dec_lo && ra >= ra_lo && ra < ra_hi {
            return c;
        }
    }
    // Unreachable: the south-cap record matches any dec >= -90, and
    // Equatorial validates dec ∈ [-90, 90].
    Constellation::Oct
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::angle::{Angle, ParseMode};

    fn j2000(ra: &str, dec: &str) -> Equatorial {
        Equatorial::parse_j2000(ra, dec, ParseMode::Strict).unwrap()
    }

    #[test]
    fn known_objects() {
        // M31 (Andromeda galaxy).
        assert_eq!(
            constellation(j2000("00:42:44.3", "+41:16:09")),
            Constellation::And
        );
        // Polaris.
        assert_eq!(
            constellation(j2000("02:31:49", "+89:15:51")),
            Constellation::UMi
        );
        // σ Octantis.
        assert_eq!(
            constellation(j2000("21:08:47", "-88:57:23")),
            Constellation::Oct
        );
        // Serpens Caput (α Ser) and Cauda (η Ser) — one constellation.
        assert_eq!(
            constellation(j2000("15:44:16", "+06:25:32")),
            Constellation::Ser
        );
        assert_eq!(
            constellation(j2000("18:21:19", "-02:53:56")),
            Constellation::Ser
        );
    }

    #[test]
    fn poles_any_ra() {
        for ra_deg in [0.0, 90.0, 123.456, 359.9] {
            let north =
                Equatorial::j2000(Angle::from_degrees(ra_deg), Angle::from_degrees(90.0)).unwrap();
            let south =
                Equatorial::j2000(Angle::from_degrees(ra_deg), Angle::from_degrees(-90.0)).unwrap();
            assert_eq!(constellation(north), Constellation::UMi);
            assert_eq!(constellation(south), Constellation::Oct);
        }
    }

    #[test]
    fn ra_wrap_pair() {
        // Both sides of the 0ʰ/24ʰ seam, well inside Pisces at J2000.
        assert_eq!(
            constellation(j2000("23:59:59.9", "+05:00:00")),
            Constellation::Psc
        );
        assert_eq!(
            constellation(j2000("00:00:00.1", "+05:00:00")),
            Constellation::Psc
        );
    }

    #[test]
    fn of_date_input_honoured() {
        let m31 = j2000("00:42:44.3", "+41:16:09");
        let of_date = precess(m31, Epoch::OfDate(2026.5));
        assert_eq!(constellation(of_date), Constellation::And);
    }

    #[test]
    fn table_walk_half_open_convention() {
        // Polar cap record (0, 24, 88, UMi): Dec lower edge is inclusive…
        assert_eq!(lookup_b1875(4.0, 88.0), Constellation::UMi);
        // …and just below it, RA 4ʰ falls through to the (0, 8, 85, Cep) zone.
        assert_eq!(lookup_b1875(4.0, 87.9999), Constellation::Cep);
        // RA edges at the 8ʰ Cep/UMi boundary (dec 87): upper exclusive,
        // lower inclusive.
        assert_eq!(lookup_b1875(7.9999, 87.0), Constellation::Cep);
        assert_eq!(lookup_b1875(8.0, 87.0), Constellation::UMi);
    }

    #[test]
    fn table_walk_wrap_and_totality() {
        // rem_euclid folds 24ʰ onto 0ʰ.
        assert_eq!(lookup_b1875(24.0, 89.0), lookup_b1875(0.0, 89.0));
        // South pole hits the final catch-all record.
        assert_eq!(lookup_b1875(12.0, -90.0), Constellation::Oct);
        // Coarse totality sweep: every grid point resolves (no panic) and the
        // set of names/abbreviations is consistent.
        let mut hits = 0usize;
        let mut dec = -90.0;
        while dec <= 90.0 {
            let mut ra = 0.0;
            while ra < 24.0 {
                let c = lookup_b1875(ra, dec);
                assert_eq!(c.abbreviation().len(), 3);
                hits += 1;
                ra += 0.5;
            }
            dec += 3.0;
        }
        assert_eq!(hits, 48 * 61);
    }

    #[test]
    fn names_and_parsing() {
        assert_eq!(Constellation::And.abbreviation(), "And");
        assert_eq!(Constellation::And.name(), "Andromeda");
        assert_eq!(Constellation::Boo.name(), "Boötes");
        assert_eq!(Constellation::Cha.name(), "Chamaeleon");
        assert_eq!(Constellation::Oph.name(), "Ophiuchus");
        assert_eq!(Constellation::PsA.name(), "Piscis Austrinus");
        assert_eq!(Constellation::UMi.to_string(), "Ursa Minor");
        assert_eq!("umi".parse::<Constellation>().unwrap(), Constellation::UMi);
        assert_eq!("UMI".parse::<Constellation>().unwrap(), Constellation::UMi);
        assert!(matches!(
            "Xyz".parse::<Constellation>(),
            Err(Error::UnknownConstellation(s)) if s == "Xyz"
        ));
    }
}
