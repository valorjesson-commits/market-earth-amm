# Mainnet Deployment Checklist
1. Multisig Setup (2-of-3 threshold).
2. Cycles Provisioning (>2T cycles).
3. Deploy `pair` first: `dfx deploy --network ic pair --no-wallet --yes`
4. Deploy `amm_factory` with the pair canister bound: `dfx deploy --network ic amm_factory --argument "(record { governance_principal = principal \"<MULTISIG_ID>\"; paused = false; whitelist = vec {}; pair_canister = principal \"<PAIR_CANISTER_ID>\" })"`
5. Verify the pair snapshot: `dfx canister --network ic call pair get_snapshot`
