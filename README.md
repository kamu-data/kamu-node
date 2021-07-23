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

## Getting Started

Install `bunyan` to get human-readable log output when running services in the foreground:

```bash
cargo install bunyan
```

To run API server using local `kamu` workspace:

```bash
cargo run -- --metadata-repo file:///home/me/workspace run | bunyan
```

To control log verbosity use the standard `RUST_LOG` env var:

```bash
RUST_LOG="trace,mio::poll=info" cargo run ...
```

To explore GQL schema run server and open http://127.0.0.1:8080/playground.

To test GQL queries from the CLI:

```bash
cargo run -- gql query '{ apiVersion }' | jq
```

## GraphQL snippets

Working with unions in search results:

```gql
{
  search {
    query(query: "foo") {
      edges {
        node {
          __typename
          ... on Dataset {
            id
          }
        }
      }
    }
  }
```
