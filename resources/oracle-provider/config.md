## `Config`

<table>
<thead><tr><th>Field</th><th>Type</th><th>Default</th><th>Description</th></tr></thead>
<tbody>
<tr>
<td><code>apiAccessToken</code></td>
<td><code>string</code></td>
<td><code class="language-json">null</code></td>
<td>API token to use for authentication with the server</td>
</tr>
<tr>
<td><code>apiUrl</code></td>
<td><code>string</code></td>
<td><code class="language-json">&quot;http:&#x2F;&#x2F;localhost:8080&#x2F;&quot;</code></td>
<td>URL of the ODF-compatible API server that will execute requests</td>
</tr>
<tr>
<td><code>blocksStride</code></td>
<td><code>integer</code></td>
<td><code class="language-json">100000</code></td>
<td>Number of blocks to examine per one getLogs RPC request when catching up</td>
</tr>
<tr>
<td><code>chainId</code></td>
<td><code>integer</code></td>
<td><code class="language-json">0</code></td>
<td>ID of the chain used during signing to prevent replay attacks</td>
</tr>
<tr>
<td><code>httpAddress</code></td>
<td><code>string</code></td>
<td><code class="language-json">&quot;127.0.0.1&quot;</code></td>
<td>Interface to listen for HTTP admin traffic on</td>
</tr>
<tr>
<td><code>httpPort</code></td>
<td><code>integer</code></td>
<td><code class="language-json">0</code></td>
<td>Port to listen for HTTP admin traffic on</td>
</tr>
<tr>
<td><code>ignoreConsumers</code></td>
<td><code>array</code></td>
<td><code class="language-json">[]</code></td>
<td>

Consumer addresses to ignore requests from (use as a disaster recovery
mechanism only)

</td>
</tr>
<tr>
<td><code>ignoreRequests</code></td>
<td><code>array</code></td>
<td><code class="language-json">[]</code></td>
<td>

Request IDs that provider should skip over (use as a disaster recovery
mechanism only)

</td>
</tr>
<tr>
<td><code>loopIdleTime</code></td>
<td><a href="#durationstring"><code>DurationString</code></a></td>
<td><code class="language-json">&quot;1s&quot;</code></td>
<td>Time to sleep while waiting for new blocks</td>
</tr>
<tr>
<td><code>oracleContractAddress</code></td>
<td><code>string</code></td>
<td></td>
<td>Address of the oracle contract to read logs from</td>
</tr>
<tr>
<td><code>providerAddress</code></td>
<td><code>string</code></td>
<td></td>
<td>Address of this provider's account to use when submitting transactions</td>
</tr>
<tr>
<td><code>providerPrivateKey</code></td>
<td><code>string</code></td>
<td></td>
<td>Private key of the provider to use when signing transactions.</td>
</tr>
<tr>
<td><code>rpcUrl</code></td>
<td><code>string</code></td>
<td><code class="language-json">&quot;http:&#x2F;&#x2F;localhost:8545&#x2F;&quot;</code></td>
<td>Ethereum-compatible JSON-RPC address</td>
</tr>
<tr>
<td><code>scanFromBlock</code></td>
<td><code>integer</code></td>
<td><code class="language-json">null</code></td>
<td>

Block number to start scanning from on startup (precedence:
scan_from_block, scan_last_blocks, scan_last_blocks_period)

</td>
</tr>
<tr>
<td><code>scanLastBlocks</code></td>
<td><code>integer</code></td>
<td><code class="language-json">null</code></td>
<td>

Number of last blocks to scan on startup (precedence: scan_from_block,
scan_last_blocks, scan_last_blocks_period)

</td>
</tr>
<tr>
<td><code>scanLastBlocksPeriod</code></td>
<td><a href="#durationstring"><code>DurationString</code></a></td>
<td><code class="language-json">null</code></td>
<td>

Time period in which blocks will be scanned on startup (precedence:
scan_from_block, scan_last_blocks, scan_last_blocks_period)

</td>
</tr>
<tr>
<td><code>transactionConfirmations</code></td>
<td><code>integer</code></td>
<td></td>
<td>Number of confirmations to await before considering transaction included</td>
</tr>
<tr>
<td><code>transactionTimeout</code></td>
<td><a href="#durationstring"><code>DurationString</code></a></td>
<td><code class="language-json">&quot;1m&quot;</code></td>
<td>Timeout when submitting a transaction</td>
</tr>
</tbody>
</table>

## `DurationString`

Base type: `string`
