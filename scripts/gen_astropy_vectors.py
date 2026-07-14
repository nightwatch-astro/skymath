#!/usr/bin/env python3

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

"""Generate AstroPy reference vectors for skymath's cross-validation suite.

Writes tests/data/astropy_vectors.json, consumed by tests/astropy_vectors.rs.
Run from the repo root:

    uv run --with astropy --with astroplan scripts/gen_astropy_vectors.py

Conventions matched to skymath:
- GMST instants are fed to AstroPy as UT1 (skymath assumes UT1 = UTC), so the
  comparison pins the IAU-1982 polynomial itself, not ΔUT1.
- Alt-az uses pressure=0 (no refraction) — skymath's alt_az is geometric.
- AstroPy's full apparent-place machinery (aberration ~20″, nutation ~17″,
  frame bias) is left ON for alt-az/hour-angle/parallactic/transit vectors;
  skymath's planning-grade contract (≤1′) absorbs it, and the Rust tolerances
  say so explicitly.
"""

import json
from pathlib import Path

import astropy
import astropy.units as u
import numpy as np
from astropy.coordinates import (
    AltAz,
    Angle,
    EarthLocation,
    FK5,
    GCRS,
    GeocentricMeanEcliptic,
    HADec,
    SkyCoord,
    get_body,
    get_constellation,
    get_sun,
)
from astropy.time import Time
from astropy.utils import iers

import astroplan

# No network at generation time; extrapolated dut1/polar motion is fine at
# planning grade.
iers.conf.auto_download = False
iers.conf.iers_degraded_accuracy = "ignore"

M31 = (10.6847, 41.2688)
M110 = (10.0921, 41.6853)
VEGA = (279.23473479, 38.78368896)
ALTAIR = (297.6958, 8.8683)
SGR_A = (266.4051, -28.9362)  # galactic-centre direction, J2000
NGP = (192.85948, 27.12825)
POLLUX = (116.328942, 28.026183)
POLARIS = (37.9546, 89.2641)

KITT_PEAK = (31.9583, -111.6, 2120.0)
LEIDEN = (52.155, 4.485, 6.0)
SIDING_SPRING = (-31.2733, 149.0644, 1165.0)


def sc(radec, frame="icrs"):
    return SkyCoord(ra=radec[0] * u.deg, dec=radec[1] * u.deg, frame=frame)


def loc(site):
    return EarthLocation(lat=site[0] * u.deg, lon=site[1] * u.deg, height=site[2] * u.m)


def iso(t):
    return t.utc.isot + "Z"


out: dict = {
    "meta": {
        "script": "scripts/gen_astropy_vectors.py",
        "astropy": astropy.__version__,
        "astroplan": astroplan.__version__,
        "numpy": np.__version__,
        "note": "instants ISO-8601 UTC; angles degrees; GMST/LST hours",
    }
}

# ── separation & position angle ────────────────────────────────────────────────
pairs = [
    (M31, M110),
    (VEGA, ALTAIR),
    ((10.0, 80.0), (200.0, 85.0)),  # over the pole
    ((359.9, -0.1), (0.1, 0.1)),  # RA wrap
    ((100.0, 0.0), (101.5, 0.5)),  # frame scale
]
out["separation_pa"] = [
    {
        "a": list(a),
        "b": list(b),
        "sep_deg": sc(a).separation(sc(b)).deg,
        "pa_deg": sc(a).position_angle(sc(b)).wrap_at(360 * u.deg).deg,
    }
    for a, b in pairs
]

# ── tangent offsets (frame-scale pairs only; definitions diverge at O(sep³)) ──
out["offsets"] = []
for a, b in [(M31, M110), ((359.9, -0.1), (0.1, 0.1)), ((100.0, 0.0), (101.5, 0.5))]:
    d_lon, d_lat = sc(a).spherical_offsets_to(sc(b))
    out["offsets"].append(
        {"a": list(a), "b": list(b), "east_deg": d_lon.deg, "north_deg": d_lat.deg}
    )

# ── apply_offset ↔ directional_offset_by ─────────────────────────────────────
out["apply_offset"] = []
for start, pa, sep in [(M31, 30.0, 1.0), ((350.0, -75.0), 200.0, 3.0)]:
    dest = sc(start).directional_offset_by(pa * u.deg, sep * u.deg)
    out["apply_offset"].append(
        {
            "from": list(start),
            "pa_deg": pa,
            "sep_deg": sep,
            "to": [dest.ra.deg, dest.dec.deg],
        }
    )

# ── precession (FK5 = IAU-1976, matching skymath's model) ─────────────────────
out["precess"] = []
for radec, jyear in [(M31, 2026.5), (M31, 1975.0), (POLARIS, 2050.0)]:
    c = SkyCoord(ra=radec[0] * u.deg, dec=radec[1] * u.deg, frame=FK5(equinox="J2000"))
    p = c.transform_to(FK5(equinox=Time(jyear, format="jyear")))
    out["precess"].append(
        {"j2000": list(radec), "epoch": jyear, "of_date": [p.ra.deg, p.dec.deg]}
    )

# ── galactic ───────────────────────────────────────────────────────────────────
out["galactic"] = []
for radec in [M31, VEGA, SGR_A, NGP]:
    g = sc(radec).galactic
    out["galactic"].append({"eq": list(radec), "l_deg": g.l.deg, "b_deg": g.b.deg})

# ── ecliptic (mean, of date) ──────────────────────────────────────────────────
out["ecliptic"] = []
for radec, when in [
    (POLLUX, "2000-01-01T12:00:00"),
    (M31, "2026-07-11T00:00:00"),
    (VEGA, "1987-04-10T00:00:00"),
]:
    e = sc(radec).transform_to(GeocentricMeanEcliptic(equinox=Time(when, scale="utc")))
    out["ecliptic"].append(
        {"eq": list(radec), "at": when + "Z", "lambda_deg": e.lon.deg, "beta_deg": e.lat.deg}
    )

# ── GMST / LST (fed as UT1: pins the IAU-1982 polynomial, not ΔUT1) ───────────
gmst_instants = [
    "1987-04-10T00:00:00",
    "1987-04-10T19:21:00",
    "2000-01-01T12:00:00",
    "2026-07-11T18:00:00",
]
out["gmst"] = [
    {
        "at": s + "Z",
        "hours": Time(s, scale="ut1").sidereal_time("mean", "greenwich", model="IAU1982").hour,
    }
    for s in gmst_instants
]
out["lst"] = [
    {
        "at": "2026-07-11T18:00:00Z",
        "lon_east_deg": site[1],
        "hours": Time("2026-07-11T18:00:00", scale="ut1")
        .sidereal_time("mean", site[1] * u.deg, model="IAU1982")
        .hour,
    }
    for site in [KITT_PEAK, LEIDEN]
]

# ── JD / MJD / Julian epoch ───────────────────────────────────────────────────
jd_instants = ["1858-11-17T00:00:00", "2000-01-01T12:00:00", "2026-07-11T22:15:03.25"]
out["jd"] = [
    {"at": s + "Z", "jd": Time(s, scale="utc").jd, "mjd": Time(s, scale="utc").mjd}
    for s in jd_instants
]
out["jyear"] = [
    {"at": "2026-07-11T00:00:00Z", "epoch": Time("2026-07-11T00:00:00", scale="utc").jyear}
]

# ── sexagesimal parsing cross-check ───────────────────────────────────────────
out["sexagesimal_ra"] = [
    {"s": "00:42:44.3", "deg": Angle("00h42m44.3s").deg},
    {"s": "17:45:37.224", "deg": Angle("17h45m37.224s").deg},
]
out["sexagesimal_dec"] = [
    {"s": "+41:16:09", "deg": Angle("+41d16m09s").deg},
    {"s": "-00:30:00", "deg": Angle("-00d30m00s").deg},
]

# ── alt-az (pressure=0, geometric) ────────────────────────────────────────────
altaz_cases = [
    (VEGA, KITT_PEAK, "2024-08-04T06:00:00"),
    (M31, LEIDEN, "2026-07-11T22:00:00"),
    ((96.0, -52.7), SIDING_SPRING, "2026-03-01T12:00:00"),  # near Canopus
]
out["alt_az"] = []
for radec, site, when in altaz_cases:
    frame = AltAz(obstime=Time(when, scale="utc"), location=loc(site), pressure=0 * u.hPa)
    h = sc(radec).transform_to(frame)
    out["alt_az"].append(
        {
            "eq": list(radec),
            "site": list(site),
            "at": when + "Z",
            "alt_deg": h.alt.deg,
            "az_deg": h.az.deg,
        }
    )

# ── hour angle ────────────────────────────────────────────────────────────────
frame = HADec(obstime=Time("2026-07-11T22:00:00", scale="utc"), location=loc(LEIDEN))
ha = sc(M31).transform_to(frame)
out["hour_angle"] = [
    {
        "eq": list(M31),
        "site": list(LEIDEN),
        "at": "2026-07-11T22:00:00Z",
        "ha_deg": ha.ha.wrap_at(180 * u.deg).deg,
    }
]

# ── parallactic angle, transit, altitude crossings (astroplan) ────────────────
observer = astroplan.Observer(location=loc(LEIDEN))
t0 = Time("2026-07-11T22:00:00", scale="utc")
m31 = astroplan.FixedTarget(sc(M31), name="M31")

out["parallactic"] = [
    {
        "eq": list(M31),
        "site": list(LEIDEN),
        "at": "2026-07-11T22:00:00Z",
        "q_deg": observer.parallactic_angle(t0, m31).to_value(u.deg),
    }
]

# Rise/set are referenced to the transit so both bracket the same window
# (astroplan's "nearest to 22:00" would otherwise pick the previous set).
transit = observer.target_meridian_transit_time(t0, m31, which="nearest")
rise = observer.target_rise_time(transit, m31, which="previous", horizon=30 * u.deg)
setting = observer.target_set_time(transit, m31, which="next", horizon=30 * u.deg)
out["transit"] = [
    {"eq": list(M31), "site": list(LEIDEN), "near": "2026-07-11T22:00:00Z", "utc": iso(transit)}
]
out["crossings"] = [
    {
        "eq": list(M31),
        "site": list(LEIDEN),
        "threshold_deg": 30.0,
        "night": "2026-07-11T22:00:00Z",
        "rise": iso(rise),
        "set": iso(setting),
    }
]

# ── 002: Sun & Moon ephemerides ───────────────────────────────────────────────
sun_instants = ["1992-10-13T00:00:00", "2026-07-11T22:00:00", "2026-10-15T23:00:00"]
out["sun"] = []
for when in sun_instants:
    s = get_sun(Time(when, scale="utc"))
    out["sun"].append({"at": when + "Z", "ra_deg": s.ra.deg, "dec_deg": s.dec.deg})

moon_instants = ["1992-04-12T00:00:00", "2026-07-11T22:00:00"]
out["moon_geocentric"] = []
for when in moon_instants:
    t = Time(when, scale="utc")
    m = get_body("moon", t).transform_to(GCRS(obstime=t))
    out["moon_geocentric"].append(
        {
            "at": when + "Z",
            "ra_deg": m.ra.deg,
            "dec_deg": m.dec.deg,
            "distance_km": m.distance.to_value(u.km),
        }
    )

out["moon_topocentric"] = []
for site_v, when in [(LEIDEN, "2026-07-11T22:00:00"), (KITT_PEAK, "2026-03-01T04:00:00")]:
    t = Time(when, scale="utc")
    location = loc(site_v)
    m = get_body("moon", t, location=location)  # topocentric GCRS
    out["moon_topocentric"].append(
        {
            "at": when + "Z",
            "site": list(site_v),
            "ra_deg": m.ra.deg,
            "dec_deg": m.dec.deg,
        }
    )

out["moon_illumination"] = [
    {"at": when + "Z", "k": float(astroplan.moon_illumination(Time(when, scale="utc")))}
    for when in ["1992-04-12T00:00:00", "2026-07-11T22:00:00", "2026-07-29T00:00:00"]
]

# Twilight for Leiden, night of 2026-10-15: evening next after 18:00 UTC, the
# following morning next after the evening instant.
leiden_obs = astroplan.Observer(location=loc(LEIDEN))
tw_ref = Time("2026-10-15T18:00:00", scale="utc")
dusk = leiden_obs.twilight_evening_astronomical(tw_ref, which="next")
dawn = leiden_obs.twilight_morning_astronomical(dusk, which="next")
out["twilight"] = [
    {
        "kind": "astronomical",
        "site": list(LEIDEN),
        "night": "2026-10-15T23:42:00Z",
        "dusk": iso(dusk),
        "dawn": iso(dawn),
    }
]

# Moonrise/set bracketing one lunar pass over Leiden.
mr = leiden_obs.moon_rise_time(Time("2026-07-11T22:00:00", scale="utc"), which="nearest")
ms = leiden_obs.moon_set_time(mr, which="next")
out["moon_crossings"] = [
    {"site": list(LEIDEN), "rise": iso(mr), "set": iso(ms)},
]

# ── 003: constellation identification ────────────────────────────────────────
# Uniform seeded sky sample + one witness per Roman (1987) zone record + curated
# probes. Sampled/witness points are kept only when AstroPy returns the same
# constellation for the point and four ±5″ offsets, so no vector sits inside
# the sub-arcsecond FK4/FK5-precession ambiguity band near boundaries (research
# R21/R24); the Rust test can then demand 100% agreement.


def stable(coords):
    """(names, abbrs, keep-mask) for an array SkyCoord, ±5″ stability probe."""
    names = np.atleast_1d(get_constellation(coords))
    abbrs = np.atleast_1d(get_constellation(coords, short_name=True))
    keep = np.ones(names.shape, dtype=bool)
    for pa in (0.0, 90.0, 180.0, 270.0):
        off = coords.directional_offset_by(pa * u.deg, 5 * u.arcsec)
        keep &= np.atleast_1d(get_constellation(off)) == names
    return names, abbrs, keep


rng = np.random.default_rng(20260712)
n_uniform = 1200
uniform = SkyCoord(
    ra=rng.uniform(0.0, 360.0, n_uniform) * u.deg,
    dec=np.degrees(np.arcsin(rng.uniform(-1.0, 1.0, n_uniform))) * u.deg,
)

# Witness candidates: zone RA midpoint, just above the zone's lower Dec edge,
# built in the table's own B1875 frame so every constellation is represented.
roman = (
    Path(str(astropy.coordinates.__file__)).parent / "data" / "constellation_data_roman87.dat"
)
w_ra_h, w_dec = [], []
for line in roman.read_text(encoding="ascii").splitlines():
    line = line.strip()
    if not line or line.startswith("#"):
        continue
    lo, hi, dlo, _code = line.split()
    w_ra_h.append((float(lo) + float(hi)) / 2.0)
    w_dec.append(min(float(dlo) + 0.1, 89.9))
witnesses = SkyCoord(
    ra=np.array(w_ra_h) * 15.0 * u.deg, dec=np.array(w_dec) * u.deg, frame=FK5(equinox="B1875")
).icrs

SIGMA_OCT = (317.1958, -88.9564)
ALPHA_SER = (236.0667, 6.4258)  # Serpens Caput
ETA_SER = (275.3275, -2.8989)  # Serpens Cauda
curated = SkyCoord(
    ra=[
        M31[0], POLARIS[0], SIGMA_OCT[0], ALPHA_SER[0], ETA_SER[0],
        0.0, 180.0, 0.0, 123.456,           # celestial poles, any RA
        359.999583, 0.000417,               # RA-wrap pair (±0.1ˢ of 0ʰ)
        135.0, 352.5, 77.25, 141.8325,      # Roman-paper check positions…
        193.332, 235.0305, 285.0, 93.333,   # …(fed as ICRS, AstroPy truth)
    ] * u.deg,
    dec=[
        M31[1], POLARIS[1], SIGMA_OCT[1], ALPHA_SER[1], ETA_SER[1],
        90.0, 90.0, -90.0, -90.0,
        5.0, 5.0,
        65.0, -20.0, -11.0, -30.0,
        22.0, -12.0, -40.0, -81.1234,
    ] * u.deg,
)

out["constellation"] = []
for coords, filtered in ((uniform, True), (witnesses, True), (curated, False)):
    names, abbrs, keep = stable(coords)
    if not filtered:
        keep[:] = True
    for c, name, abbr, ok in zip(coords, names, abbrs, keep):
        if ok:
            # .strip(): astropy's names file carries a trailing space ("Crux ").
            out["constellation"].append(
                {
                    "ra_deg": round(c.ra.deg, 6),
                    "dec_deg": round(c.dec.deg, 6),
                    "name": str(name).strip(),
                    "abbr": str(abbr).strip(),
                }
            )

covered = {case["name"] for case in out["constellation"]}
if len(covered) != 88:
    raise SystemExit(f"constellation coverage incomplete: {len(covered)}/88")
print(f"constellation cases: {len(out['constellation'])} (all 88 covered)")

# Epoch-of-date input: M31 expressed at equinox J2026.5 must still identify as
# Andromeda (input epoch honoured via precession to B1875).
m31_of_date = sc(M31).transform_to(FK5(equinox=Time(2026.5, format="jyear")))
out["constellation_of_date"] = [
    {
        "ra_deg": m31_of_date.ra.deg,
        "dec_deg": m31_of_date.dec.deg,
        "epoch_jyear": 2026.5,
        "name": "Andromeda",
    }
]

dest = Path(__file__).resolve().parent.parent / "tests" / "data" / "astropy_vectors.json"
dest.parent.mkdir(parents=True, exist_ok=True)
dest.write_text(json.dumps(out, indent=1) + "\n")
print(f"wrote {dest} (astropy {astropy.__version__}, astroplan {astroplan.__version__})")
