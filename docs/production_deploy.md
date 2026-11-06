# Mainnet Deployment Checklist

1. **Governance setup**
   - Use a multisig principal for `governance_principal`.
   - Keep canister controller permissions minimal.

2. **Build artifacts**
   - `cargo test --workspace`
   - `cargo build --workspace --target wasm32-unknown-unknown --release`

3. **Deploy token canisters**
   - Deploy your production token canisters first (not `mock_token`).
   - Confirm `icrc1_transfer` and `icrc2_transfer_from` are enabled.

4. **Deploy pair canister**
   - Install `pair` with `(token_a_principal, token_b_principal)`.
   - Verify `get_pair_info`, `get_reserves`, and `quote_swap` respond.

5. **Deploy factory canister**
   - Install `amm_factory`:
     ```bash
     dfx deploy --network ic amm_factory --argument \
       "(record { governance_principal = principal \"<MULTISIG_ID>\"; paused = false; whitelist = vec {} })"
     ```

6. **Register pair in factory**
   - Governance principal calls:
     ```bash
     dfx canister --network ic call amm_factory create_pair \
       "(principal \"<TOKEN_A>\", principal \"<TOKEN_B>\", principal \"<PAIR_CANISTER>\")"
     ```

7. **Post-deploy verification**
   - `dfx canister --network ic call amm_factory list_pairs`
   - Approve pair on both tokens (`icrc2_approve`) from a funded wallet.
   - Execute small `add_liquidity`, `quote_swap`, and `swap` smoke checks.
