# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2020-09-19
### Initial Release

## [0.1.5] - 2020-10-08
### Added
- Multithreading for the main update function

## [0.1.6] - 2020-10-30
### Added
- Internal data structures and trait implementations for error handling

### Changed
- Internal error handling and code layout to prevent future crashes
- Cogsy from the CLI now exits with an error if something goes wrong
- Updating from the CLI now prints to stdout properly