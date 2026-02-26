<div align="center">

<img alt="kamu - planet-scale data pipeline" src="docs/readme_files/kamu_logo.png" width=300/>

<p>

[![Release](https://img.shields.io/github/v/release/kamu-data/kamu-node?include_prereleases&logo=rust&logoColor=orange&style=for-the-badge)](https://github.com/kamu-data/kamu-node/releases/latest)
[![CI](https://img.shields.io/github/actions/workflow/status/kamu-data/kamu-node/build.yaml?logo=githubactions&label=CI&logoColor=white&style=for-the-badge&branch=master)](https://github.com/kamu-data/kamu-node/actions)
[![Chat](https://shields.io/discord/898726370199359498?style=for-the-badge&logo=discord&label=Discord)](https://discord.gg/nU6TXRQNXC)

[![Docs](https://img.shields.io/static/v1?logo=gitbook&logoColor=66BBFF&label=&message=Docs&color=gray&style=for-the-badge)](https://docs.kamu.dev/node/)
[![REST API](https://img.shields.io/static/v1?logo=openapiinitiative&logoColor=6BFF39&label=&message=REST%20API&color=gray&style=for-the-badge)](https://docs.kamu.dev/node/api/rest/)
[![GraphQL API](https://img.shields.io/static/v1?logo=graphql&logoColor=F133A8&label=&message=GraphQL%20API&color=gray&style=for-the-badge)](https://api.demo.kamu.dev/graphql)

</p>
</div>

## About

Kamu Node is a set of [Kubernetes](https://kubernetes.io/)-native applications that can be deployed in any cloud or on-prem to:

- Operate the stream processing pipelines for a certain set of data flows
- Continuously verify datasets that you are interested it to catch malicious behavior
- Execute queries on co-located data

Nodes are the building pieces of the [Open Data Fabric](https://www.opendatafabric.org/) and the primary way of contributing resources to the network. Unlike blockchain nodes that maintain a single ledger, Kamu nodes can form loosely connected clusters based on vested interests of their operators in certain data pipelines.

If you are new to ODF - we recommend you to start with [Kamu CLI](https://github.com/kamu-data/kamu-cli/) for a gradual introduction.

You should consider Kamu Node when you want to:
- Build a horizontally-scalable lakehouse for your data
- Need a decentralized infrastructure for sharing data with your partners or globally without intermediaries
- Want to continuously operate ODF data pipelines or verify data
- Need a rich set of [data APIs](https://docs.kamu.dev/node/protocols/)
- Want to provide data to [ODF blockchain oracle](https://docs.kamu.dev/node/protocols/oracle/)


## API Server
Prerequisites:
* Install `rustup`

To run API server using local `kamu` workspace:

```bash
# 1. Create a configuration file
{
  echo 'repo:'
  echo '  repoUrl: workspace/.kamu/datasets'
  echo 'datasetEnvVars:'
  echo '  enctyptionKey: QfnEDcnUtGSW2pwVXaFPvZOwxyFm2BOC'
} > config.yaml
# 2. Run
cargo run --bin kamu-api-server -- --config config.yaml run

# Alternative: pass the repo url via env:
KAMU_API_SERVER_CONFIG_repo__repoUrl=workspace/.kamu/datasets kamu-api-server run
```

To control log verbosity use the standard `RUST_LOG` env var:

```bash
RUST_LOG="trace,mio::poll=info" cargo run ...
```

To explore GQL schema run server and open http://127.0.0.1:8080/playground.

To test GQL queries from the CLI:

```bash
cargo run --bin kamu-api-server -- gql query '{ apiVersion }' | jq
```


### API Server with Remote Repository (S3 bucket)

To use it:

```bash
# 1. Create a configuration file
{
  echo 'repo:'
  echo '  repoUrl: s3://example.com/kamu_repo'
} > config.yaml
# 2. Run
cargo run --bin kamu-api-server -- --config config.yaml run
```


### GitHub Auth
To use API server for GitHub's OAuth, you need to set the following configuration settings:

```yaml
auth:
  providers:
    - kind: github
      clientId: CLIENT_ID_OF_YOUR_GITHUB_OAUTH_APP
      clientSecret: CLIENT_SECRET_OF_YOUR_GITHUB_OAUTH_APP
```

Then you can use the following mutation:

```gql
mutation GithubLogin {
  auth {
    githubLogin (code: "...") {
      token {
        accessToken
        scope
        tokenType
      }
      accountInfo {
        login
        email
        name
        avatarUrl
        gravatarId
      }
    }
  }
}
```
