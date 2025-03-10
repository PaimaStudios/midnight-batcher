## Building

```
git clone ...
git submodule update --init --recursive
```

The build.rs file does modify the submodule in order to use only path
dependencies. This is done to avoid running into compilation issues with
duplicated git dependencies.

## Local chain usage

1. Setup the local chain and fund the batcher.

```sh
cd ./local-chain-setup
source .envrc
npm install
docker compose up
# in another terminal
npm run fund-batcher
```

2. Run the batcher.

```sh
cd ./local-chain-setup
source .envrc
cargo run --release
```

## Whitelisting

The `--allowed-contract` flag has to be used to constrain the batcher to a
single contract. The validation consists on checking that the deploy call
has the smae operation names (exported circuits in compact) as the expected
contract, and with the same verifier keys.

Example:

```
cargo run --release -- --allowed-contract ~/Work/pvp-arena/examples/pvp/contract/dist/managed/pvp/keys
```

**NOTE:** Compact doesn't remove old circuits from the keys directory (if
circuits are renamed or deleted), and this will cause errors.

## Server config

For the server configuration refer to the [Rocket documentation](https://rocket.rs/guide/v0.4/configuration/).
