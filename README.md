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

Web UI requires a running API Server, so you should either follow the above steps to set it up, or, if you are not planning to do any backend development, you can get the latest version with the following command:

```bash
# Get the latest version
docker pull kamudata/api-server:latest-with-data

# Run with example data
docker run -it --rm -p 8080:8080 kamudata/api-server:latest-with-data

# Run with a local kamu workspace
docker run -it --rm -p 8080:8080 -v /my/workspace:/opt/kamu/workspace:ro kamudata/api-server:latest-with-data
```

Once you have the API server running, you can start Web UI in development mode:

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
