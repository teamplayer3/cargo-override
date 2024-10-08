# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.5](https://github.com/eopb/cargo-override/compare/v0.0.4...v0.0.5) - 2024-09-14

### Added

- Support for Windows, tested in CI

### Fixed

- Escaping of paths which was causing panics on Windows ([#134](https://github.com/eopb/cargo-override/pull/134))
- Typos in error messages ([#143](https://github.com/eopb/cargo-override/pull/143))

### Other

- Some dependency upgrades

## [0.0.4](https://github.com/eopb/cargo-override/compare/v0.0.3...v0.0.4) - 2024-09-11

### Added

- Implement `--force` flag for ignoring correctness and compatibility checks ([#127](https://github.com/eopb/cargo-override/pull/127))

## [0.0.3](https://github.com/eopb/cargo-override/compare/v0.0.2...v0.0.3) - 2024-09-08

### Fixed

- Missing styling in CLI help text

### Other

- Introduced nix flake installation method
- Started running tests on MacOS in CI

## [0.0.2](https://github.com/eopb/cargo-override/compare/v0.0.1...v0.0.2) - 2024-09-04

### Other
- First release with a changelog 🚀
