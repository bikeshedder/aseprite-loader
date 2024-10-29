# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.2] - 2024-10-29

### Fixed

- Fix index out of bounds error in `combined_frame_image` code when
  cels overlap are moved outside the visible bounds of a frame.

## [0.3.1] - 2024-10-09

### Changed

- Make `AsepriteFile` fields public
- Make `loader::decompress` function public

### Fixed

- Fix hashing implementation by using the `DefaultHasher` instead of a
  custom implementation.

## [0.3.0] - 2024-07-11

### Changed

- Change `CelType::Unknown` to include the `cel_type`
- Make `Fixed` fields public
- Replace `Range` by `InclusiveRange` to avoid overflow errors
- Update `itertools` to `0.13`

## [0.2.0] - 2024-04-20

### Added

- Add support for cel blending

### Removed

- Rename `sprity-aseprite` crate to `aseprite-loader`
- Remove `sprity` crate

## [0.1.2] - 2023-12-02

### Added

- Add `repeat` field to animation tags
- Add `File::slices` field
- Add support for custom properties
- Add support for z-index property of cels
- Add support for xflip, yflip and dflip cel flags

## [0.1.1] - 2023-12-01

### Fixed

- Remove unused dependencies

## [0.1.0] - 2023-12-01

### Added

- Initial release

[unreleased]: https://github.com/bikeshedder/aseprite-loader/compare/v0.3.2...HEAD
[0.3.2]: https://github.com/bikeshedder/aseprite-loader/compare/sprity-aseprite-v0.3.1...v0.3.2
[0.3.1]: https://github.com/bikeshedder/aseprite-loader/compare/sprity-aseprite-v0.3.0...v0.3.1
[0.3.0]: https://github.com/bikeshedder/aseprite-loader/compare/sprity-aseprite-v0.2.0...v0.3.0
[0.2.0]: https://github.com/bikeshedder/aseprite-loader/compare/sprity-aseprite-v0.1.2...v0.2.0
[0.1.2]: https://github.com/bikeshedder/aseprite-loader/compare/sprity-aseprite-v0.1.1...sprity-aseprite-v0.1.2
[0.1.1]: https://github.com/bikeshedder/aseprite-loader/compare/sprity-aseprite-v0.1.0...sprity-aseprite-v0.1.1
[0.1.0]: https://github.com/bikeshedder/aseprite-loader/releases/tag/sprity-aseprite-v0.1.0
