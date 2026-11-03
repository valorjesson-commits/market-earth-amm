# Market Earth AMM Mainnet Deployment Guide

## Prerequisites

- `dfx` installed and configured
- Rust toolchain with `wasm32-unknown-unknown`
- Governance multisig principal
- Sufficient cycles for all canisters

## 1) Build and test

```bash
cargo test --workspace
cargo build --workspace --target wasm32-unknown-unknown --release
```

## 2) Deploy production token canisters

Deploy your token canisters first. The pair canister expects both tokens to support:
- `icrc1_transfer`
- `icrc2_transfer_from`

## 3) Deploy pair canister

```bash
dfx deploy --network ic pair --argument \
  "(principal \"<TOKEN_A>\", principal \"<TOKEN_B>\")"
```

## 4) Deploy factory canister

```bash
dfx deploy --network ic amm_factory --argument \
  "(record { governance_principal = principal \"<MULTISIG_ID>\"; paused = false; whitelist = vec {} })"
```

## 5) Register pair in factory

```bash
dfx canister --network ic call amm_factory create_pair \
  "(principal \"<TOKEN_A>\", principal \"<TOKEN_B>\", principal \"<PAIR_CANISTER>\")"
```

## 6) Verify

```bash
dfx canister --network ic call amm_factory get_config
dfx canister --network ic call amm_factory list_pairs
dfx canister --network ic call pair get_pair_info
dfx canister --network ic call pair get_reserves
```

## 7) Smoke-test liquidity and swap

From a funded account:
1. `icrc2_approve` both tokens to the pair canister
2. Call `add_liquidity`
3. Call `quote_swap`
4. Execute a small `swap`
