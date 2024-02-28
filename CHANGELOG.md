# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
