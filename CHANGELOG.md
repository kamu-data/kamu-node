# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.18.2] - 2024-04-09
### Fixed
- Fixed CI build

## [0.18.1] - 2024-04-09
### Changed
- Updated to `kamu v0.171.1` to correct the web link in the Data access panel

## [0.18.0] - 2024-04-05
### Changed
- Updated to `kamu v0.171.0` to put in place endpoints for the Data Access panel

## [0.17.1] - 2024-03-26
### Fixed
- Enable local FS object store for push ingest to work

## [0.17.0] - 2024-03-23
### Added
- Made number of runtime threads configurable
### Changed
- Incorporate FlightSQL performance fixes in `kamu v0.168.0`

## [0.16.3] - 2024-03-23
### Fixed
- Incorporate FlightSQL location bugfix in `kamu-adapter-flight-sql v0.167.2`

## [0.16.2] - 2024-03-20
### Fixed
- Incorporate dataset creation handle bugfix in `kamu-core v0.167.1`

## [0.16.1] - 2024-03-18
### Fixed
- Changed config env var prefix to `KAMU_API_SERVER_CONFIG_` to avoid collisions with Kubernetes automatic variables

## [0.16.0] - 2024-03-18
### Added
- Support for metadata object caching on local file system (e.g. to avoid too many calls to S3 repo)
- Support for caching the list of datasets in a remote repo (e.g. to avoid very expensive S3 bucket prefix listing calls)

## [0.15.1] - 2024-03-14
### Fixed
- OData adapter will ignore fields with unsupported data types instead of crashing

## [0.15.0] - 2024-03-14
### Added
- Experimental support for [OData](https://odata.org) protocol

## [0.14.1] - 2024-03-13
### Changed
- Updated to `kamu v0.165.0` to bring in flow system latest demo version

## [0.14.0] - 2024-03-06
### Changed
- Updated to `kamu v0.164.0` to bring in new REST data endpoints

## [0.13.3] - 2024-02-29
### Added
- Introduced a `ghcr.io/kamu-data/kamu-api-server:latest-with-data-mt` image with multi-tenant workspace

## [0.13.2] - 2024-02-28
### Change
- Updated to `kamu v0.162.1` to bring in more verbose logging on JWT token rejection reason

## [0.13.1] - 2024-02-28
### Fixed
- Startup crash in Flow Service that started to require admin token to operate

## [0.13.0] - 2024-02-28
### Changed
- Updated to `kamu v0.162.0`

## [0.12.2] - 2024-02-13
### Changed
- Upgraded Rust toolchain and minor dependencies
- Synced with `kamu` v0.158.0

## [0.12.0] - 2024-01-17
### Changed
- Upgraded to major changes in ODF and `kamu`

## [0.11.0] - 2023-11-26
### Added
- Push ingest API

## [0.10.0] - 2023-10-29
### Added
- Introduced a config file allowing to configure the list of supported auth providers

## [0.9.0] - 2023-10-26
### Added
- FlightSQL endpoint

## [0.8.0] - 2023-10-13
### Added
- Integrated multi-tenancy support: authentication & authorization for public datasets

## [0.7.0] - 2023-07-27
### Added
- Keeping a CHANGELOG
### Changed
- Integrated latest core with engine I/O strategies - this allows `api-server` run ingest/transform tasks for datasets located in S3 (currently by downloading necessary inputs locally)
