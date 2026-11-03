# Deployment Instructions

## Local deployment (recommended first)

```bash
GOV_PRINCIPAL="$(dfx identity get-principal)" ./deploy_local.sh
./scripts/integration_liquidity_test.sh
```

This deploys:
- `token_a` (`mock_token`)
- `token_b` (`mock_token`)
- `pair`
- `amm_factory`

and registers the pair in the factory.

## Mainnet flow

1. Deploy your real token canisters.
2. Deploy `pair` with token principals.
3. Deploy `amm_factory` with governance principal.
4. Register the pair:

```bash
dfx canister --network ic call amm_factory create_pair \
  "(principal \"<TOKEN_A>\", principal \"<TOKEN_B>\", principal \"<PAIR_CANISTER>\")"
```

5. Verify:

```bash
dfx canister --network ic call amm_factory list_pairs
dfx canister --network ic call pair get_pair_info
```
