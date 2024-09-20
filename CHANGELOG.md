# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.39.0] - 2024-09-20
### Changed
Upgrade kamu-cli version to `0.202.0`:
  - Major dependency upgrades:
    - DataFusion 42
    - HTTP stack v.1
    - Axum 0.7
    - latest AWS SDK
    - latest versions of all remaining libs we depend on
  - Outbox refactoring towards true parallelism via Tokio spaned tasks instead of futures
### Fixed
- Re-enabled missing optional features for eth, ftp, mqtt ingest and JSON SQL extensions
- Failed flows should still propagate `finishedAt` time
- Eliminate span.enter, replaced with instrument everywhere

## [0.38.0] - 2024-09-19
### Added
- REST API: New `/verify` endpoint allows verification of query commitment
### Changed
- Upgrade kamu-cli version to `0.201.0`:
  - Outbox main loop was revised to minimize the number of transactions
  - Detecting concurrent modifications in flow and task event stores
  - Improved and cleaned handling of flow abortions at different stages of processing
  - Revised implementation of flow scheduling to avoid in-memory time wheel

## [0.37.1] - 2024-09-13
### Changed
- Added application name prefix to Prometheus metrics

## [0.37.0] - 2024-09-13
### Added
- API Server now exposes Prometheus metrics
- FlightSQL tracing
### Changed
- Oracle Provider Prometheus metrics names changed to conform to the convention

## [0.36.0] - 2024-09-10
### Changed
- Oracle Provider: Updated to use V2 `/query` REST API
- Oracle Provider: Added ability to scan back only a certain interval of past blocks
- Oracle Provider: Added ability to ignore requests by ID and from certain consumers

## [0.35.2] - 2024-09-09
### Fixed
- Identity config registration bug that prevented response signing from working

## [0.35.1] - 2024-09-09
### Added
- REST API: The `/query` endpoint now supports response proofs via reproducibility and signing (#816)
- REST API: New `/{dataset}/metadata` endpoint for retrieving schema, description, attachments etc. (#816)
### Changed
- Upgrade kamu-cli version to `0.199.2`
  - Hot fixes in persistent Tasks & Flows

## [0.35.0] - 2024-09-06
### Changed
- Upgrade kamu-cli version to `0.199.1`
  - Persistent Tasks & Flows
  - Database schema breaking changes
  - Get Data Panel: use SmTP for pull & push links 
  - GQL api method `setConfigCompaction` allows to set `metadataOnly` configuration for both root and derived datasets
  - GQL api `triggerFlow` allows to trigger `HARD_COMPACTION` flow in `metadataOnly` mode for both root and derived datasets  

## [0.34.1] - 2024-08-30
### Fixed
- Critical errors were not logged due to logging guard destroyed before the call to tracing

## [0.34.0] - 2024-08-30
### Changed
- Upgrade kamu-cli version to `0.198.2`
  - ReBAC: in-memory & SQLite components
  - Smart Transfer Protocol: breaking changes

## [0.33.0] - 2024-08-27
### Changed
- Upgrade kamu-cli version to `0.198.0` (address [RUSTSEC-2024-0363](https://rustsec.org/advisories/RUSTSEC-2024-0363))

## [0.32.1] - 2024-08-22
### Fixed
- Add missed `ResetService` dependency

## [0.32.0] - 2024-08-22
### Changed
- Upgrade kamu-cli version to `0.197.0`

## [0.31.1] - 2024-08-16
### Fixed
- Missing initialization issue for outbox processor

## [0.31.0] - 2024-08-16
### Changed
- Upgrade kamu-cli version to 0.195.1 (DataFusion 41, Messaging outbox)

## [0.30.0] - 2024-08-13
### Changed
- Upgrade kamu-cli version to 0.194.0 and add `DatasetKeyValueSysEnv` service if encryption key was not provided

## [0.29.2] - 2024-07-30
### Changed
- Upgrade kamu-cli version to 0.191.5 and add init of new `DatasetKeyValueService` in catalog 

## [0.29.1] - 2024-07-23
### Changed
- Exposed new `engine`, `source`, and `protocol` sections in the `api-server` config (#109)

## [0.29.0] - 2024-07-23
### Changed
- Dropped "bunyan" log format in favor of standard `tracing` JSON logs (#106)
### Added
- The `oracle-provider` now exposes Prometheus metrics via `/system/metrics` endpoint (#106)
- All apps now support exporting traces via Open Telemetry protocol (#106)
- The `api-server` now support graceful shutdown (#106)
- All apps now support `/system/health?type={liveness,readiness,startup}` heath check endpoint using Kubernetes probe semantics (#106)

## [0.28.3] - 2024-07-23
### Changed
- Make dataset env vars encryption key optional

## [0.28.2] - 2024-07-22
### Changed
- Upgraded to kamu `0.191.4`

## [0.28.1] - 2024-07-22
### Changed
- Upgraded to kamu `0.191.3`

## [0.28.0] - 2024-07-19
### Added
- Integrated the `DatasetEnvVars` service that allows configuring custom variables and secrets to be used during the data ingestion
### Changed
- Upgraded to new `rustc` version and some dependencies
- Upgraded to kamu `0.191.2`

## [0.27.4] - 2024-07-11
### Fixed
- Regression where oracle provider stopped respecting the `block_stride` config

## [0.27.3] - 2024-07-08
### Fixed
- Upgraded to kamu `0.189.7` which fixes the operation of SmTP along with database transactions

## [0.27.0] - 2024-06-28
### Added
- Integrating modes of RDS password access

## [0.26.4] - 2024-06-24
### Fixed
- Upgraded to kamu `0.188.3` which is fixing file ingestion feature

## [0.26.3] - 2024-06-20
### Fixed
- Upgraded to kamu `0.188.1` that includes a fix for transactions getting "stuck" in data queries

## [0.26.2] - 2024-06-18
### Fixed
- Fixed invalid REST response decoding by `oracle-provider`

## [0.26.1] - 2024-06-18
### Fixed
- Fixed invalid REST request encoding by `oracle-provider`

## [0.26.0] - 2024-06-18
### Changed
- Upgraded to kamu `0.188.1`
- Improve `oracle-provider`:
  - Dataset identity support
  - SQL errors and missing dataset handling
  - Reproducibility state support

## [0.25.1] - 2024-06-14
### Changed
- Upgraded to kamu `0.188.0`

## [0.25.0] - 2024-06-14
### Changed
- Oracle provider was migrated from deprecated `ethers` to `alloy` crate
- Upgraded to kamu `0.186.0`

## [0.24.0] - 2024-06-10
### Changed
- Upgraded `kamu` from `0.181.1` to `0.185.1` ([changelog](https://github.com/kamu-data/kamu-cli/blob/master/CHANGELOG.md))

## [0.23.3] - 2024-05-20
### Fixed
- Hotfix: upgrade to Kamu CLI v0.181.1 (dealing with unresolved accounts)

## [0.23.2] - 2024-05-17
### Fixed
- HTTP API: add `/platform/login` handler to enable GitHub authorization inside Jupyter Notebook

## [0.23.1] - 2024-05-17
### Fixed
- Fix startup: correct config parameter name (`jwt_token` -> `jwt_secret`)

## [0.23.0] - 2024-05-14
### Changed
- Upgraded `kamu` from `0.177.0` to `0.180.0` ([changelog](https://github.com/kamu-data/kamu-cli/blob/master/CHANGELOG.md))
- Read settings from config file, absorb:
  - `--repo-url` CLI argument
  - environment variables used for configuration

## [0.22.0] - 2024-05-02
### Added
- Introduced new `kamu-oracle-provider` component which can fulfil data requests from any EVM compatible blockchain, working in conjunction with `OdfOracle` contracts defined in [`kamu-contracts`](https://github.com/kamu-data/kamu-contracts/) repository

## [0.21.0] - 2024-04-26
### Changed
- Upgraded `kamu` from `0.176.3` to `0.177.0` ([changelog](https://github.com/kamu-data/kamu-cli/blob/master/CHANGELOG.md))
- CI improvements:
  - use `cargo-udeps` to prevent the possibility of using unused dependencies
  - use `cargo-binstall` to speed up CI jobs

## [0.20.3] - 2024-04-21
### Fixed
- Missing compacting service dependency

## [0.20.2] - 2024-04-18
### Changed
- Synchronized with latest `kamu-cli` v0.176.3

## [0.20.1] - 2024-04-17
### Fixed
- Fixed startup failure by missed DI dependency

## [0.20.0] - 2024-04-15
### Changed
- The `/ingest` REST API endpoint also supports event time hints via odf-event-time header
### Fixed
- Removed paused from `setConfigCompacting` mutation
- Extended GraphQL `FlowDescriptionDatasetHardCompacting` empty result with a resulting message
- GraphQL Dataset Endpoints object: fixed the query endpoint

## [0.19.0] - 2024-04-09
### Added
- OData API now supports querying by collection ID/key (e.g. `account/covid.cases(123)`)
### Fixed
- Fixed all pedantic lint warnings

## [0.18.3] - 2024-04-09
### Changed
- Fixed CI build

## [0.18.2] - 2024-04-09
### Fixed
- Updated to `kamu v0.171.2` to correct the CLI push command in the Data access panel

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
