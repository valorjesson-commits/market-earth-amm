# Market Earth AMM

Internet Computer AMM canisters with local integration tests and CI validation.

## What is implemented

- **`amm_factory`**: governance-controlled pair registry (`create_pair`, `remove_pair`, pause, whitelist).
- **`pair`**: constant-product AMM pool with:
  - `add_liquidity`
  - `remove_liquidity`
  - `swap`
  - `quote_swap`
  - LP accounting
  - ICRC transfer calls (`icrc1_transfer`, `icrc2_transfer_from`).
- **`mock_token`**: local ICRC-like test token canister used by integration tests.

## Quick start (local)

**Prerequisite:** `dfx` 0.24+ (0.32.0 validated). Older versions use the legacy
ic-ref emulator, which cannot run modern Rust WASM.

```bash
# 1) Deploy local replica + token_a/token_b/pair/factory
GOV_PRINCIPAL="$(dfx identity get-principal)" ./deploy_local.sh

# 2) Run integration flow (approve -> add liquidity -> quote -> swap -> remove liquidity)
./scripts/integration_liquidity_test.sh
```

## Contract notes

- Factory pair creation is **governance-only** and now requires the deployed pair canister principal:

```candid
create_pair(token_a : principal, token_b : principal, pair_canister : principal)
```

- Factory token whitelist is optional; if set, both tokens must be present.
- Pair uses a **0.30% fee** (`30 / 10_000`) in swap quotes/execution.

## CI

- Rust unit tests (`cargo test --workspace`)
- WASM build checks for all canisters
- Local DFX integration workflow (`deploy_local.sh` + `integration_liquidity_test.sh`)

## Deployment

See [docs/production_deploy.md](docs/production_deploy.md) for the updated production checklist.
