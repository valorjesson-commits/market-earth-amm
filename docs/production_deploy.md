# Mainnet Deployment Checklist
1. Multisig Setup (2-of-3 threshold).
2. Cycles Provisioning (>2T cycles).
3. Deploy command: `dfx deploy --network ic amm_factory --argument "(record { governance_principal = principal \"<MULTISIG_ID>\"; paused = false; whitelist = vec {} })"`
