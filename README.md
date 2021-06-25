# Kamu Platform

## Developer Instructions

```bash
# API server is outputing traces to stdout in JSON format
# For human-readable logs during development install `bunyan` and pipe output into it
cargo install bunyan
cargo run | bunyan

# To control log verbosity use the standard `RUST_LOG` env var
RUST_LOG="trace,mio::poll=info" cargo run | bunyan
```


## TODO
- repo
- CLI commands (schema)
- CI
  - schema check
- tie with ODF
  - SHA
