# cw-cyph
A key-value storage smart contract implementation in Rust using [CosmWasm](https://github.com/CosmWasm/cosmwasm). This smart contract allows you to add, delete, and edit key-value pairs using a smart contract in the Cosmos ecosystem.

# Usage
This build process assumes you have `wasmd` and Rust installed on your machine. Instructions on how to set up your `wasmd` wallets can be found [here](https://docs.cosmwasm.com/docs/1.0/getting-started/setting-env). 

1. Install dependencies:
    ```shell
    rustup default stable
    ```
2. In the `cw-cyph` compile the wasm contract:
    ```shell
    RUSTFLAG='-C link-args=-s' cargo wasm
    ```
3. Run unit tests:
    ```shell
    RUST_BACKTRACE=1 cargo unit-test
    ```
4. Run the `rust-optimizer` using docker:
    ```shell
    docker run --rm -v "$(pwd)":/code \
        --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
        --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
        cosmwasm/rust-optimizer:0.12.4
    ```
5. Use `wasmd` to upload te contract:
    ```shell
    RES=$(wasmd tx wasm store artifacts/cw_nameservice.wasm --from wallet $TXFLAG -y --output json)
    ```
6. Get the contract code ID and instantiate it:
    ```shell
    CODE_ID=$(echo $RES | jq -r '.logs[0].events[-1].attributes[0].value')
    INIT='{}'
    wasmd tx wasm instantiate $CODE_ID "$INIT" \
    --from wallet --label "cyph password manager" $TXFLAG -y
    ```

More information on how to interact with the contract can be found [here](https://docs.cosmwasm.com/docs/1.0/getting-started/interact-with-contract).