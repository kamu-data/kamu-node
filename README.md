<div align="center">
  <h1>Kamu Platform</h1>
  <p>
    <strong>World's first decentralized data warehouse</strong>
  </p>
  <p>

[![build](https://github.com/kamu-data/kamu-platform/workflows/build/badge.svg)](https://github.com/kamu-data/kamu-platform/actions)
[![Release](https://github.com/kamu-data/kamu-platform/workflows/release/badge.svg)](https://github.com/kamu-data/kamu-platform/actions)

  </p>
</div>

## Developer Instructions

```bash
# API server is outputing traces to stdout in JSON format
# For human-readable logs during development install `bunyan` and pipe output into it
cargo install bunyan
cargo run | bunyan

# To control log verbosity use the standard `RUST_LOG` env var
RUST_LOG="trace,mio::poll=info" cargo run -- run | bunyan

# To test the GQL API:
# Either start the server and navigate to http://localhost:8080/playground
# Or use the CLI command
cargo run -- gql query '{ apiVersion }' | jq
```
