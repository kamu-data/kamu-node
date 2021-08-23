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

### API Server

Prerequisites:
* Install `rustup`
* Install `bunyan` crate (`cargo install bunyan`) to get human-readable log output when running services in the foreground

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

### Web UI

Prerequisites:
* Install `nvm`
* Install latest `nodejs` (`nvm install node`)
* Fetch dependencies (`cd web-ui; npm install`)

To run Web UI in development mode first start the API server and then do:

```bash
npm run dev -- --open
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
}
```
