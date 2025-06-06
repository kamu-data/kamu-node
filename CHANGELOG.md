# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!--
Recommendation: for ease of reading, use the following order:
- Added
- Changed
- Fixed
-->

## [0.69.0] - 2025-06-06
### Added
- Support renaming accounts via GraphQL API ([kamu CLI `0.241.0`](https://github.com/kamu-data/kamu-cli/releases/tag/v0.241.0)):
  - automatically actualizes dataset entries (denormalized account names)
  - automatically updates alias files in ODF storage layer
  - properly handling account renames when it's initiated by updates to predefined configuration
### Fixed
- Missing length validation for webhook subscription labels.
- Unexpected webhook label duplication for empty labels.


## [0.68.1] - 2025-06-04
### Fixed
- Wallet-based authentication: interactive login use case support (Device Flow) ([kamu CLI `0.240.1`](https://github.com/kamu-data/kamu-cli/releases/tag/v0.240.1)).

## [0.68.0] - 2025-06-02
### Added
- Wallet-based authentication, Phase 1 ([kamu CLI `0.240.0`](https://github.com/kamu-data/kamu-cli/releases/tag/v0.240.0)).

## [0.67.0] - 2025-05-26
### Added
- Webhooks prototype ([kamu CLI `0.239.0`](https://github.com/kamu-data/kamu-cli/releases/tag/v0.239.0))

## [0.66.0] - 2025-05-22
### Added
- Account Deletion API ([kamu CLI `0.238.0`](https://github.com/kamu-data/kamu-cli/releases/tag/v0.238.0))

## [0.65.0] - 2025-05-14
### Added
- New REST endpoint `/system/info` and GQL endpoint `buildInfo` that return application version and build details
### Changed
- Upgraded to `datafusion v47.0.0` and latest `arrow`, `object-store`, and `flatbuffers` versions

## [0.64.0] - 2025-04-24
### Added
- Device Flow Authentication ([kamu CLI `0.235.0`](https://github.com/kamu-data/kamu-cli/releases/tag/v0.235.0))

## [0.63.0] - 2025-04-23
### Changed
- Pinned version for `aws-sdk-s3` crate version before breaking changes
- Update all minor versions of other crates
- Denormalization: DatasetEntry now contains a copy of owner's account name for faster dataset handle resolutions without extra round trip to database
- Speedup of account flow runs listing

## [0.62.3] - 2025-04-22
### Changed
- Forcing warmup of dataset ids listing cache on startup

## [0.62.2] - 2025-04-21
### Fixed
- Reverted `sqlx` upgrade that breaking image build

## [0.62.1] - 2025-04-19
### Changed
- Outbox: Added new param in consumer metadata `initial_consumer_boundary` which allow new consumer to not process all messages, but start from latest one
### Fixed
- S3 get_stored_dataset_by_id operation takes advantage of in-memory datasets listing cache

## [0.62.0] - 2025-04-09
### Added
- Automatically indexing key dataset blocks in the database for quicker navigation:
   - indexing all previously stored datasets at startup
   - indexing new changes to datasets incrementally, whenever HEAD advances
- Metadata chain visiting algorithm can now use the key blocks cached in the database
   to efficiently implement iteration over key blocks, when data events are not needed
### Changed
- E2E, `kamu-node-e2e-repo-tests`: remove a `kamu-api-server` dependency 
    that did not cause the `kamu-api-server` binary to be rebuilt.
- Upgraded to latest version of `dill=0.13`.

## [0.61.1] - 2025-04-08
### Added
- New value `semantic_search_threshold_score` in search configuration which used in `UiConfiguration`

## [0.61.0] - 2025-04-08
### Added
- New `engine.datafusionEmbedded` config section allows to pass custom DataFusion settings 
    when engine is used in ingest, batch query, and compaction contexts.
- GQL: 
  - `Datasets::role()`: returns the current user's role in relation to the dataset
  - GQL: `DatasetsMut::create_empty()` & `DatasetsMut::create_from_snapshot()`: alias validation in multi-tenant mode.
### Changed
- GQL: `DatasetsMut::create_empty()` & `DatasetsMut::create_from_snapshot()`: `dataset_visibility` is now mandatory.
- `kamu push/pull` command with `--force` flag now does not allow overwriting of seed block
### Fixed
- Multiple performance improvements in batch queries to avoid unnecessary metadata scanning.

## [0.60.1] - 2025-03-31
### Fixed 
- Correct api version

## [0.60.0] - 2025-03-31
### Changed
- Trigger dependent dataset flows on Http `/ingest` and on smart transfer protocol dataset push
- `DatasetSummary` files replaced with `DatasetStatistics` stored in the database 
    and updated synchronously with the HEAD reference updates
- Statistics is automatically pre-computed for all existing datasets on first use
- `DatasetHandle` and `DatasetEntry` now contain dataset kind marker
- Provenance service and pull request planner fetch dependencies from the graph
- Implemented caching layer for `DatasetEntry` within the currently open transaction

## [0.59.0] - 2025-03-25
### Added
- Private Datasets: sharing access ([kamu CLI `0.230.0`](https://github.com/kamu-data/kamu-cli/releases/tag/v0.230.0))

## [0.58.0] - 2025-03-23
### Added
- Introduced `debug` CLI command group for operational helpers
- New `debug semsearch-reindex` CLI command that allows to recreate the embeddings in the vector repository
- Semantic search:
  - More configuration options for indexing, allowing to skip datasets without descriptions or no data.
  - Overfetch is now configurable
  - Service will make repeated queries to the vector store to fill the requested results page size.
### Changed
- Flow: Updated the `BatchingRule` trigger to accept 0 for both properties(`min_records_to_await` and `max_batching_interval`), enabling dependency flow execution even when no data is added to the root dataset.
### Fixed
- HTTP & GQL API: Fixed internal error when query contains an unknown column

## [0.57.0] - 2025-03-19
### Added
- E2E: running tests also for S3 repositories
- DB-backed dataset references: they are now stored in the database, supporting transactional updates
- Ensured short transaction length in ingest & transform updates and compaction tasks.  
- Dataset Reference indexing to build the initial state of the dataset references.  
- Implemented in-memory caching for dataset references that works within the current 
### Changed
- Replaced default GraphQL playground with better maintained `graphiql` (old playground is still available)
- Improved API server web console looks
- Upgraded to `datafusion v46` (#1146)
- Dependency graph updates are improved for transactional correctness.  
- Extracted ODF dataset builders for LFS and S3 to allow for custom implementations.  
### Fixed
- Flow progress notifier is now more resilient to deleted datasets
- Use actual `base_url` in catalog configuration instead default one
- REST API: `GET /datasets/{id}` returns account data as it should 
- If dataset creation is interrupted before a dataset entry is written, 
   such a dataset is ignored and may be overwritten

## [0.56.1] - 2025-03-14
### Fixed
- Value of flow agent throttling now presented in seconds

## [0.56.0] - 2025-03-13
### Added
- Prometheus metrics: S3 ([kamu CLI `0.226.5`](https://github.com/kamu-data/kamu-cli/releases/tag/v0.226.5))
- New `FlowSystemConfig` structure in `CLIConfig` which allows to configure `flow_agent` and `task_agent` services with next options `awaiting_step_secs` and `mandatory_throttling_period_secs`
- GQL: New natural language search API - to use this feature you'll need to configure the OpenAI API key and a [Qdrant](https://qdrant.tech/) vector database connection

## [0.55.1] - 2025-03-07
### Fixed
- Fix regression with substitution of incorrect `ServerUrlConfig` component

## [0.55.0] - 2025-03-07
### Changed
- Upgrade kamu-cli version to `0.226.4`

## [0.54.3] - 2025-02-20
### Fixed
- OData API: fixed crash when accessing private dataset ([kamu CLI `0.225.0`](https://github.com/kamu-data/kamu-cli/releases/tag/v0.225.0))

## [0.54.2] - 2025-02-20
### Fixed
- Restoring OData API tolerance to trailing slashes (kamu CLI `0.223.1`)

## [0.54.1] - 2025-02-14
### Fixed
- Added access token notifier registration

## [0.54.0] - 2025-02-13
### Added
- New access token notifier that sends an email to the user each time a new access token is created (kamu CLI `0.222.0`)
### Changed
- Upgrade kamu-cli version to `0.223.0`
  - Upgraded to `datafusion v45`
  - Upgraded to `axum v0.8`

## [0.53.1] - 2025-02-03
### Fixed
- Private Datasets: hot-fixes (kamu CLI `0.221.1`)

## [0.53.0] - 2025-01-30
### Added
- Integration of email gateway (Postmark):
   - Defined `EmailSender` crate and prototyped `Postmark`-based implementation
   - Config support for gateway settings (API key, sender address & name)
   - Applied `askoma` templating engine and defined base HTML-rich template for all emails
   - Emails are fire-and-forget / best effort
   - First emails implemented: account registration, flow failed
- GQL support to query and update email on the currently logged account
### Changed
- Emails are mandatory for Kamu accounts now:
   - predefined users need to specify an email in config
   - predefined users are auto-synced at startup in case they existed before
   - GitHub users are queried for primary verified email, even if it is not public
   - migration code for the database existing users

## [0.52.1] - 2025-01-29
### Fixed
- Corrected access rights checks for transport protocols (SiTP, SmTP)

## [0.52.0] - 2025-01-28
### Changed
- Core changes from the Private Datasets epic (kamu CLI `0.220.0`), Vol. 2

## [0.51.0] - 2025-01-20
### Changed
- Toolchain updated to `nightly-2024-12-26`
- Core changes from the Private Datasets epic (kamu CLI `0.219.1`)

## [0.50.4] - 2024-01-15
### Fixed
- Telemetry-driven fixes in flow listings (kamu CLI `0.217.3`)

## [0.50.3] - 2024-01-13
### Changed
- Batched loading of flows and tasks (kamu CLI `0.217.2`)

## [0.50.2] - 2024-01-09
### Changed
- Extend database config (kamu CLI `0.217.1`)

## [0.50.1] - 2024-01-09
### Changed
- Add missing `RemoteStatusServiceImpl` service to catalog

## [0.50.0] - 2024-01-09
### Changed
- Env var and flow API changes (kamu CLI `0.217.0`)

## [0.49.0] - 2024-12-30
### Changed
- Flight SQL authentication (see https://github.com/kamu-data/kamu-cli/pull/1012)

## [0.48.1] - 2024-12-30
### Changed
- `/verify` endpoint hot fix (kamu CLI `0.215.1`)

## [0.48.0] - 2024-12-27
### Changed
- Flow configuration separation (kamu CLI `0.215.0`)

## [0.47.0] - 2024-12-23
### Changed
- Improved FlightSQL session state management (kamu CLI `0.214.0`)

## [0.46.3] - 2024-12-21
### Fixed
- Regression in FlightSQL interface related to database-backed `QueryService`

## [0.46.2] - 2024-12-19
### Fixed
- Less aggressive telemetry for key dataset services, like ingestion (kamu CLI `0.213.1`)

## [0.46.1] - 2024-12-18
### Fixed
- Eliminated regression crash on metadata queries

## [0.46.0] - 2024-12-18
### Changed
- Upgrade kamu-cli version to `0.213.0`
  - Upgrade to `datafusion v43`
  - Upgrade to `alloy v0.6`
  - Planners and executors in key dataset manipulation services
### Fixed
- Environment variables are automatically deleted if the dataset they refer to is deleted.

## [0.45.0] - 2024-12-03
### Changed
- Upgrade kamu-cli version to `0.211.0`:
  - Dataset dependency graph is now backed with a database, removing need in dependency scanning at startup.

## [0.44.0] - 2024-11-29
### Changed
- Upgrade kamu-cli version to `0.210.0`:
  - Improved OpenAPI integration
  - Replaced Swagger with Scalar for presenting OpenAPI spec
  - `kamu-api-server`: error if specialized config is not found
  - Separated runtime and dynamic UI configuration (such as feature flags)

## [0.43.1] - 2024-11-22
### Changed
- Upgrade kamu-cli version to `0.208.1` (minor updates in data image)

## [0.43.0] - 2024-11-22
### Changed
- Introduced `DatasetRegistry` abstraction, encapsulating listing and resolution of datasets (kamu-cli version to `0.208.0`):
  - Registry is backed by database-stored dataset entries, which are automatically maintained
  - Scope for `DatasetRepository` is now limited to support `DatasetRegistry` and in-memory dataset dependency graph
  - New concept of `ResolvedDataset`: a wrapper around `Arc<dyn Dataset>`, aware of dataset identity
  - Query and Dataset Search functions now consider only the datasets accessible for current user
  - Core services now explicitly separate planning (transactional) and execution (non-transactional) processing phases
  - Similar decomposition introduced in task system execution logic
  - Batched form for dataset authorization checks
  - Ensuring correct transactionality for dataset lookup and authorization checks all over the code base
  - Passing multi/single tenancy as an enum configuration instead of boolean
  - Renamed outbox "durability" term to "delivery mechanism" to clarify the design intent

## [0.42.3] - 2024-11-22
### Fixed
- Upgrade kamu-cli version to `0.207.3` (Outbox versions)

## [0.42.2] - 2024-11-14
### Fixed
- Upgrade kamu-cli version to `0.207.1`

## [0.42.1] - 2024-11-12
### Fixed
- Correct image version

## [0.42.0] - 2024-11-12
### Changed
- Upgrade kamu-cli version to `0.207.0`

## [0.41.3] - 2024-10-30
### Changed
- Upgrade kamu-cli version to `0.206.5`

## [0.41.2] - 2024-10-28
### Changed
- Upgrade kamu-cli version to `0.206.3`:
  - GraphQL: Removed deprecated `JSON_LD` in favor of `ND_JSON` in `DataBatchFormat`
  - GraphQL: In `DataBatchFormat` introduced `JSON_AOS` format to replace the now deprecated JSON in effort to harmonize format names with REST API
### Fixed
- GraphQL: Fixed invalid JSON encoding in `PARQUET_JSON` schema format when column names contain special character
- Improved telemetry for dataset entry indexing process
- Corrected recent migration related to outbox consumptions of old dataset events

## [0.41.1] - 2024-10-24
### Changed
- Upgrade kamu-cli version to `0.206.1`:
  - `DatasetEntryIndexer`: guarantee startup after `OutboxExecutor` for a more predictable initialization 
    - Add `DatasetEntry`'is re-indexing migration

## [0.41.0] - 2024-10-23
### Added
- Introduced OpenAPI spec generation
  - `/openapi.json` endpoint now returns the generated spec
  - `/swagger` endpoint serves an embedded Swagger UI for viewing the spec directly in the running server
  - OpenAPI schema is available in the repo `resources/openapi.json` beside its multi-tenant version
- Added endpoint to read a recently uploaded file (`GET /platform/file/upload/{upload_token}`)
### Changed
- Upgrade kamu-cli version to `0.205.0`:
  - Simplified organization of startup initialization code over different components
  - Postgres implementation for dataset entry and account Re-BAC repositories
  - `DatasetEntry` integration that will allow us to build dataset indexing
  - Added REST API endpoint:
    - `GET /info`
    - `GET /accounts/me`
    - `GET /datasets/:id`

## [0.40.1] - 2024-09-24
### Changed
- Upgrade kamu-cli version to `0.203.1`:
  - Added database migration & scripting to create an application user with restricted permissions

## [0.40.0] - 2024-09-22
### Added
- Support `List` and `Struct` arrow types in `json` and `json-aoa` encodings

## [0.39.0] - 2024-09-20
### Changed
- Upgrade kamu-cli version to `0.202.0`:
  - Major dependency upgrades:
    - DataFusion 42
    - HTTP stack v.1
    - Axum 0.7
    - latest AWS SDK
    - latest versions of all remaining libs we depend on
  - Outbox refactoring towards true parallelism via Tokio spanned tasks instead of futures
### Fixed
- Re-enabled missing optional features for eth, ftp, mqtt ingest and JSON SQL extensions
- Failed flows should still propagate `finishedAt` time
- Eliminate `span.enter`, replaced with instrument everywhere

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
