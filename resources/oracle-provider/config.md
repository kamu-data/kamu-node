## `Config`

| Field | Type | Required | Description |
|---|---|---|---|
| `apiAccessToken` | `string` |  | API token to use for authentication with the server |
| `apiUrl` | `string` |  | URL of the ODF-compatible API server that will execute requests |
| `blocksStride` | `integer` |  | Number of blocks to examine per one getLogs RPC request when catching up |
| `chainId` | `integer` |  | ID of the chain used during signing to prevent replay attacks |
| `httpAddress` | `string` |  | Interface to listen for HTTP admin traffic on |
| `httpPort` | `integer` |  | Port to listen for HTTP admin traffic on |
| `ignoreConsumers` | `array` |  | Consumer addresses to ignore requests from (use as a disaster recovery<br>mechanism only) |
| `ignoreRequests` | `array` |  | Request IDs that provider should skip over (use as a disaster recovery<br>mechanism only) |
| `loopIdleTime` | [`DurationString`](#durationstring) |  | Time to sleep while waiting for new blocks |
| `oracleContractAddress` | `string` | `V` | Address of the oracle contract to read logs from |
| `providerAddress` | `string` | `V` | Address of this provider's account to use when submitting transactions |
| `providerPrivateKey` | `string` | `V` | Private key of the provider to use when signing transactions. |
| `rpcUrl` | `string` |  | Ethereum-compatible JSON-RPC address |
| `scanFromBlock` | `integer` |  | Block number to start scanning from on startup (precedence:<br>scan_from_block, scan_last_blocks, scan_last_blocks_period) |
| `scanLastBlocks` | `integer` |  | Number of last blocks to scan on startup (precedence: scan_from_block,<br>scan_last_blocks, scan_last_blocks_period) |
| `scanLastBlocksPeriod` | [`DurationString`](#durationstring) |  | Time period in which blocks will be scanned on startup (precedence:<br>scan_from_block, scan_last_blocks, scan_last_blocks_period) |
| `transactionConfirmations` | `integer` | `V` | Number of confirmations to await before considering transaction included |
| `transactionTimeout` | [`DurationString`](#durationstring) |  | Timeout when submitting a transaction |


## `DurationString`

Base type: `string`
