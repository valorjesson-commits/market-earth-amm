#!/usr/bin/env bash
set -euo pipefail
GOV_PRINCIPAL="${GOV_PRINCIPAL:-aaaaa-aa}"
rustup target add wasm32-unknown-unknown
cargo build --manifest-path canisters/pair/Cargo.toml --target wasm32-unknown-unknown --release
cargo build --manifest-path canisters/amm_factory/Cargo.toml --target wasm32-unknown-unknown --release
dfx stop || true
dfx start --background --clean
for i in $(seq 1 15); do dfx canister list >/dev/null 2>&1 && break || sleep 2; done
dfx deploy --no-wallet amm_factory --argument "(record { governance_principal = principal \"$GOV_PRINCIPAL\"; paused = false; whitelist = vec {} })"
