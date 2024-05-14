# Developer Guide <!-- omit in toc -->

Please see [Kamu CLI's Developer Guide](https://github.com/kamu-data/kamu-cli/blob/master/DEVELOPER.md) for general setup instructions as this repo follows the same conventions.

## Oracle Provider Tests
Testing Oracle Provider requires a few more dependencies to work with smart contracts.

You can exclude Oracle tests by running `make test-no-oracle`, but to get all tests running:

Install [`foundry`](https://book.getfoundry.sh/getting-started/installation) - the tool we use to develop smart contracts. On Linux it's as simple as:
```sh
curl -L https://foundry.paradigm.xyz | bash
```

Check out [`kamu-contracts`](https://github.com/kamu-data/kamu-contracts) side by side with `kamu-node` directory.

Initialize the repo's `npm` dependencies:
```sh
cd kamu-contracts
nvm use
npm ci
```

You should can now run `make test`.
