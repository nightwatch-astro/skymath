#!/usr/bin/env python3
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
    GeocentricMeanEcliptic,
    HADec,
    SkyCoord,
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

dest = Path(__file__).resolve().parent.parent / "tests" / "data" / "astropy_vectors.json"
dest.parent.mkdir(parents=True, exist_ok=True)
dest.write_text(json.dumps(out, indent=1) + "\n")
print(f"wrote {dest} (astropy {astropy.__version__}, astroplan {astroplan.__version__})")
