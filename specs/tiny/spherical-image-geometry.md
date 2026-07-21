# TinySpec: Spherical Image Geometry

**Branch**: feat/image-orientation-transport
**Date**: 2026-07-21
**Status**: done
**Complexity**: small

## What

Add spherical primitives that let image-matching consumers express sky orientation and project sky positions into one tangent plane. Image footprints and matching policy remain outside `skymath`.

## Context

| File | Role |
|------|------|
| `src/coords.rs` | Add orientation transport and gnomonic projection APIs |
| `src/lib.rs` | Re-export the public APIs |
| `tests/properties.rs` | Add round-trip and transport properties |

## Requirements

1. `transport_position_angle` parallel-transports an east-of-north position angle along the unique shortest great-circle arc and returns a normalized angle.
2. Transport preserves the input angle at identical positions and returns `None` for antipodal or numerically near-antipodal positions.
3. `GnomonicPoint` stores dimensionless east and north tangent-plane coordinates.
4. `gnomonic_project` projects positions in the open hemisphere centred on the tangent point and returns `None` at or beyond its horizon.
5. `gnomonic_unproject` maps finite tangent-plane coordinates back to the centre's epoch and rejects non-finite coordinates.
6. Unit, property, and documentation tests cover identity, equatorial transport, tangent arrival, reverse transport, RA wrap, high declination, poles, antipodes, projection origin, round trips, horizon rejection, and near-horizon stability.
7. The change adds no image-footprint, WCS, parity, mount, session, mosaic, or threshold policy.

## Plan

1. Add shortest-arc parallel transport and gnomonic projection primitives to `coords`.
2. Re-export the new public types and functions from the crate root.
3. Add deterministic and property-based tests for the specified domains and failure cases.
4. Run formatting, Clippy with warnings denied, all tests, doctests, and API documentation.

## Tasks

- [x] Implement and document orientation transport.
- [x] Implement and document typed gnomonic projection and inverse projection.
- [x] Re-export the public API.
- [x] Add deterministic and property tests.
- [x] Pass all quality gates.

## Done When

- [x] All tasks are checked off.
- [x] Formatting and Clippy pass with no warnings.
- [x] Unit, integration, property, and documentation tests pass.
- [x] API documentation builds.
