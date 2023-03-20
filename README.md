<div align="center">

<img alt="kamu - planet-scale data pipeline" src="docs/readme_files/kamu_logo.png" width=300/>

<p>

[![Release](https://img.shields.io/github/v/release/kamu-data/kamu-platform?include_prereleases&logo=rust&logoColor=orange&style=for-the-badge)](https://github.com/kamu-data/kamu-platform/releases/latest)
[![CI](https://img.shields.io/github/actions/workflow/status/kamu-data/kamu-platform/build.yaml?logo=githubactions&label=CI&logoColor=white&style=for-the-badge&branch=master)](https://github.com/kamu-data/kamu-platform/actions)
[![Chat](https://shields.io/discord/898726370199359498?style=for-the-badge&logo=discord&label=Discord)](https://discord.gg/nU6TXRQNXC)

</p>
</div>


## API Server
Prerequisites:
* Install `rustup`
* Install `bunyan` crate (`cargo install bunyan`) to get human-readable log output when running services in the foreground

To run API server using local `kamu` workspace:

```bash
cargo run -- --local-repo /home/me/workspace/.kamu run | bunyan
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


### Synchronization with Remote Repository
Until we implement support for operating over remote storage API server has a temporary option that will continuously monitor a remote repository (e.g. S3 bucket) and will sync all changes into local workspace. This only works for read-only synchronization and all changes to local workspace will be overwritten upon every sync.

To use it:

```bash
cargo run -- --local-repo /tmp/kamu_local_repo --repo-url s3://example.com/kamu_repo run | bunyan
```


### GitHub Auth
To use API server for GitHub's OAuth you will need to set following environment variables:
- `KAMU_AUTH_GITHUB_CLIENT_ID` - Client ID of your GitHub OAuth app
- `KAMU_AUTH_GITHUB_CLIENT_SECRET` - Client secret of your GitHub OAuth app

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
