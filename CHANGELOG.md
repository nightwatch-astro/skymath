# Changelog

## [0.5.0](https://github.com/nightwatch-astro/skymath/compare/v0.4.0...v0.5.0) (2026-07-17)


### Features

* lenient J2000 constructor that normalizes out-of-range coordinates ([#27](https://github.com/nightwatch-astro/skymath/issues/27)) ([b1cbe17](https://github.com/nightwatch-astro/skymath/commit/b1cbe17b40cad38a8eb18c4875783af41b779216)), closes [#21](https://github.com/nightwatch-astro/skymath/issues/21)
* sexagesimal HMS/DMS component accessors ([#28](https://github.com/nightwatch-astro/skymath/issues/28)) ([850be3f](https://github.com/nightwatch-astro/skymath/commit/850be3f9dc1e328ac179a19cb0080ecbfc3ddef3)), closes [#22](https://github.com/nightwatch-astro/skymath/issues/22)


### Bug Fixes

* stop CLA lock from breaking release-please, add recovery publish path ([#24](https://github.com/nightwatch-astro/skymath/issues/24)) ([c2674a9](https://github.com/nightwatch-astro/skymath/commit/c2674a9ee8a8dd1dc5a2fbddf0b86b94a7b95ffa))

## [0.4.0](https://github.com/nightwatch-astro/skymath/compare/v0.3.4...v0.4.0) (2026-07-17)


### ⚠ BREAKING CHANGES

* relicense from Apache-2.0 to MPL-2.0 ([#16](https://github.com/nightwatch-astro/skymath/issues/16))

### Features

* circular mean and circular distance for angles ([#20](https://github.com/nightwatch-astro/skymath/issues/20)) ([8ecccd2](https://github.com/nightwatch-astro/skymath/commit/8ecccd2931bd48c98c47eb260ec0524cea27be11))


### Bug Fixes

* accept trailing Z UTC designator in parse_date_obs ([#19](https://github.com/nightwatch-astro/skymath/issues/19)) ([38feda5](https://github.com/nightwatch-astro/skymath/commit/38feda5c77cdf75c96f037e93181ff68a4b4500f))
* store CLA signatures on unprotected branch, allowlist owner ([#23](https://github.com/nightwatch-astro/skymath/issues/23)) ([4167be1](https://github.com/nightwatch-astro/skymath/commit/4167be1461e70f6466de8890133d2137314e22b1))
* use GitHub App token for CLA bot instead of PAT ([#18](https://github.com/nightwatch-astro/skymath/issues/18)) ([076efe5](https://github.com/nightwatch-astro/skymath/commit/076efe5be37273d2fef37a73d5f75566122613a3))


### Miscellaneous Chores

* relicense from Apache-2.0 to MPL-2.0 ([#16](https://github.com/nightwatch-astro/skymath/issues/16)) ([943747a](https://github.com/nightwatch-astro/skymath/commit/943747ad4995448100ccdec0f2e8848d1765095d))

## [0.3.4](https://github.com/nightwatch-astro/skymath/compare/v0.3.3...v0.3.4) (2026-07-13)


### Documentation

* use absolute URLs for guide and example links so they resolve on docs.rs ([#14](https://github.com/nightwatch-astro/skymath/issues/14)) ([8802637](https://github.com/nightwatch-astro/skymath/commit/8802637b6722caf6f89de5cf633484164f6332b8))

## [0.3.3](https://github.com/nightwatch-astro/skymath/compare/v0.3.2...v0.3.3) (2026-07-13)


### Documentation

* add status badges and link the guide on docs.rs ([#12](https://github.com/nightwatch-astro/skymath/issues/12)) ([7cc02a5](https://github.com/nightwatch-astro/skymath/commit/7cc02a5498dbe1ec6414188b78ef4b2f87c1424e))

## [0.3.2](https://github.com/nightwatch-astro/skymath/compare/v0.3.1...v0.3.2) (2026-07-13)


### Documentation

* make the crate root render the README and surface the guide ([#10](https://github.com/nightwatch-astro/skymath/issues/10)) ([5e3be82](https://github.com/nightwatch-astro/skymath/commit/5e3be82b1894280934218e115f04eee696c87d80))

## [0.3.1](https://github.com/nightwatch-astro/skymath/compare/v0.3.0...v0.3.1) (2026-07-13)


### Bug Fixes

* crate-root docs.rs links now resolve ([#8](https://github.com/nightwatch-astro/skymath/issues/8)) ([9e8873c](https://github.com/nightwatch-astro/skymath/commit/9e8873cde63e56764fc8f9d8fbf0716085f6b422))

## [0.3.0](https://github.com/nightwatch-astro/skymath/compare/v0.2.0...v0.3.0) (2026-07-12)


### Features

* identify the IAU constellation containing any sky coordinate ([#6](https://github.com/nightwatch-astro/skymath/issues/6)) ([a3ba312](https://github.com/nightwatch-astro/skymath/commit/a3ba312e5e9c71272fc4f63525059cd055a09801))

## [0.2.0](https://github.com/nightwatch-astro/skymath/compare/v0.1.0...v0.2.0) (2026-07-12)


### Features

* Sun and Moon ephemerides — twilight, lunar separation, illumination, moon-avoidance ([#4](https://github.com/nightwatch-astro/skymath/issues/4)) ([cb9d63a](https://github.com/nightwatch-astro/skymath/commit/cb9d63a9b5e5a9dbb107448697dfe5a08ec67e4b))

## 0.1.0 (2026-07-12)


### Features

* planning-grade astronomy math library (coordinates, time, observer, frames) ([#1](https://github.com/nightwatch-astro/skymath/issues/1)) ([b9dd3f7](https://github.com/nightwatch-astro/skymath/commit/b9dd3f70ddaaa5706662d793dfe9d972728b9a95))
