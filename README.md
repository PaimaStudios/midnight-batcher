## Building


```sh
git clone ...
git submodule update --init --recursive
```

Then (only once)

```sh
pip install toml # or any preferred of getting the dependency
python scripts/patch-midnight-ledger-prototype-with-local-deps.py
```

The script modifies the `Cargo.toml` files inside the submodule so that
they use local paths to the other dependencies in there, instead of using
url dependencies to the same monorepo. This is done to avoid running into
compilation issues with duplicated git dependencies, and it also makes patching
easier.

## Local chain usage

1. Setup the local chain and fund the batcher.

```sh
cd ./local-chain-setup
source .envrc
npm install
docker compose up
```

2. Run the batcher.

```sh
cd ./local-chain-setup
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
