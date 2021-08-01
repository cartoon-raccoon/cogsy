# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [0.2.2] - 2021-07-26

### Added
- Additional `update` subcommand options for selective updates
- Added the ability to update from a CSV file

### Changes
- Internal refactoring

### Fixed
- Typo in `reset` option for `database` subcommand

## [0.2.1] - 2021-05-03

### Added
- User formatting for collection display text
- Sorting releases by different orders

### Changed
- Removed "reset" subcommand - replaced with "database" for general database administration
- Specific handling for orphan table errors

## [0.2.0] - 2021-03-11
### Added
- Ok button on album info page
- History button on album info page
- Colored text in message box
- Customizable colours for some screen elements
- Quit command (to quit from commandline)

### Changed
- A lot of internal refactoring
- Drastic changes to configuration format
- Verbose output actually does something now

### Fixed
- Panic when not providing a value to in-app update

## [0.1.12] - 2021-03-02
### Fixed
- Fixed a bug where integrity check would fail on empty folders

## [0.1.11] - 2021-03-01
### Changed
- More informative integrity check errors

### Fixed
- Verbose output not showing unimplemented message

## [0.1.10] - 2021-03-01
### Added
- Verbose option to update (not yet doing anything though)

### Fixed
- A bug that caused a database error when there were duplicate entries in the user's collection

## [0.1.9] - 2021-02-27
### Added
- Colour customization in configuration file

### Changed
- More helpful initialization failure procedure

### Fixed
- Fixed reset never being called when app initialization fails

## [0.1.8] - 2021-02-26
### Added
- 'Listen' button inside release info page
- `reset` subcommand

### Changed
- Tiny UI change - narrower folder window in collection view
- Internal code refactoring

## [0.1.7] - 2020-12-22
### Changed
- Fixed a bug that caused an error when loading folders with spaces in their name

## [0.1.6] - 2020-10-30
### Added
- Internal data structures and trait implementations for error handling

### Changed
- Internal error handling and code layout to prevent future crashes
- Cogsy from the CLI now exits with an error if something goes wrong
- Updating from the CLI now prints to stdout properly

## [0.1.5] - 2020-10-08
### Added
- Multithreading for the main update function

## [0.1.0] - 2020-09-19
### Initial Release