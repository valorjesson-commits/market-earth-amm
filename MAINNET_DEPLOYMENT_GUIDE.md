# Market Earth AMM - Mainnet Deployment Guide

## Prerequisites

1. **DFX CLI**: Install from [internetcomputer.org](https://internetcomputer.org/docs/current/developer-docs/setup/install/)
2. **Rust**: `rustup target add wasm32-unknown-unknown`
3. **GitHub Secrets**: Configure `GOV_PRINCIPAL` in your repository
4. **ICP Wallet**: Fund with cycles (>2 trillion for safe deployment)
5. **Multisig Setup**: Recommended 2-of-3 threshold for governance

## Step 1: Configure GitHub Secrets

1. Go to **Settings → Secrets and variables → Actions**
2. Create secret `GOV_PRINCIPAL` with your multisig principal ID
3. Example: `f7ffu-3qaaa-aaaaa-aaaba-cai`

## Step 2: Prepare Canisters

```bash
# Switch to canister-implementation branch
git checkout canister-implementation

# Build canisters
rustup target add wasm32-unknown-unknown
cargo build --manifest-path canisters/amm_factory/Cargo.toml --target wasm32-unknown-unknown --release
cargo build --manifest-path canisters/pair/Cargo.toml --target wasm32-unknown-unknown --release
```

## Step 3: Deploy to Mainnet

```bash
# Set your governance principal
export GOV_PRINCIPAL="f7ffu-3qaaa-aaaaa-aaaba-cai"

# Deploy AMM Factory
dfx deploy --network ic amm_factory --argument "(record { 
  governance_principal = principal \"$GOV_PRINCIPAL\"; 
  paused = false; 
  whitelist = vec {} 
})"

# Get factory canister ID
FACTORY_ID=$(dfx canister --network ic id amm_factory)
echo "Factory deployed at: $FACTORY_ID"
```

## Step 4: Verify Deployment

```bash
# Check factory status
dfx canister --network ic status $FACTORY_ID

# Get factory configuration
dfx canister --network ic call $FACTORY_ID get_config

# List pairs (initially empty)
dfx canister --network ic call $FACTORY_ID list_pairs
```

## Step 5: Create First Pair

```bash
# Example token principals (replace with actual token IDs)
TOKEN_A="ryjl3-tyaaa-aaaaa-aaaba-cai"
TOKEN_B="r7inp-6aaaa-aaaaa-aaaaa-cai"

# Create pair
dfx canister --network ic call $FACTORY_ID create_pair "(principal \"$TOKEN_A\", principal \"$TOKEN_B\")"
```

## Step 6: Add Liquidity

```bash
# Get pair canister ID from create_pair output
PAIR_ID="<pair-canister-id>"

# Approve tokens (on token canisters)
dfx canister --network ic call $TOKEN_A approve "(principal \"$PAIR_ID\", 1_000_000_000 : nat)"
dfx canister --network ic call $TOKEN_B approve "(principal \"$PAIR_ID\", 1_000_000_000 : nat)"

# Add liquidity
dfx canister --network ic call $PAIR_ID add_liquidity "(500_000_000 : nat, 500_000_000 : nat)"

# Verify reserves
dfx canister --network ic call $PAIR_ID get_reserves
```

## Step 7: Test Swap

```bash
# Perform a swap
dfx canister --network ic call $PAIR_ID swap "(
  principal \"$TOKEN_A\", 
  10_000 : nat, 
  1 : nat
)"
```

## Governance Operations

### Pause Factory (Only Governance)

```bash
dfx canister --network ic call $FACTORY_ID set_paused "(true)"
```

### Unpause Factory

```bash
dfx canister --network ic call $FACTORY_ID set_paused "(false)"
```

## CI/CD Pipeline

The GitHub Actions workflow automatically:
1. Builds Rust canisters on each push to `main` or `amm-deploy-setup`
2. Runs integration tests
3. Performs E2E liquidity operations

## Troubleshooting

### Insufficient Cycles
```
Error: Canister operation failed with error: Insufficient cycles
```
→ Increase cycles allocation via `dfx canister deposit-cycles`

### Unauthorized Operation
```
Error: Unauthorized
```
→ Verify caller is the governance principal

### Pair Already Exists
```
Error: Pair already exists
```
→ Use `list_pairs` to find existing pair ID

## Security Checklist

- [ ] Governance principal is a multisig
- [ ] All token contracts are whitelisted/verified
- [ ] Liquidity providers understand impermanent loss
- [ ] Smart contract audit completed
- [ ] Emergency pause mechanism tested
- [ ] Cycles buffer maintained (>500B)

## Support

For issues, refer to:
- [Internet Computer Docs](https://internetcomputer.org/docs/)
- [dfx CLI Reference](https://internetcomputer.org/docs/current/references/dfx-cli/)
