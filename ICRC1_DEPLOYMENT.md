# ICRC-1 Compliant DEX Deployment Guide

Complete step-by-step guide to deploy Market Earth with ICRC-1 token canisters and updated pair/factory for ICPSwap integration.

---

## Prerequisites

```bash
# Ensure you have:
dfx --version                  # 0.14.0+
rustc --version               # 1.70+
mops --version                # Latest

# Set your IC identity
dfx identity use default
dfx identity get-principal     # Note this for governance
```

---

## Step 1: Build and Generate Fresh Canisters

### 1.1 Clean Previous Builds
```bash
cd market-earth-amm
cargo clean
rm -rf .dfx
```

### 1.2 Create Canister IDs on Mainnet

```bash
# Create Me-Coin canister (ICRC-1 token)
dfx canister create me_coin --network ic
ME_COIN_ID=$(dfx canister id me_coin --network ic)
echo "Me-Coin Canister ID: $ME_COIN_ID"

# Create We-Coin canister (ICRC-1 token)
dfx canister create we_coin --network ic
WE_COIN_ID=$(dfx canister id we_coin --network ic)
echo "We-Coin Canister ID: $WE_COIN_ID"

# Create Pair canister (LP token + swaps)
dfx canister create pair --network ic
PAIR_ID=$(dfx canister id pair --network ic)
echo "Pair Canister ID: $PAIR_ID"

# Create Factory canister (pool registry)
dfx canister create amm_factory --network ic
FACTORY_ID=$(dfx canister id amm_factory --network ic)
echo "Factory Canister ID: $FACTORY_ID"
```

### 1.3 Build Rust Canisters

```bash
# Build all Rust canisters
cargo build --target wasm32-unknown-unknown --release \
  --manifest-path canisters/amm_factory/Cargo.toml

cargo build --target wasm32-unknown-unknown --release \
  --manifest-path canisters/pair/Cargo.toml
```

**Verification:**
```bash
ls -lh target/wasm32-unknown-unknown/release/
# Should show: amm_factory.wasm, pair.wasm
```

---

## Step 2: Install ICRC-1 Token Code

### 2.1 Deploy Me-Coin (ICRC-1 Token)

```bash
# Get your principal (governance account)
GOVERNANCE_PRINCIPAL=$(dfx identity get-principal)
echo "Governance Principal: $GOVERNANCE_PRINCIPAL"

# Deploy Me-Coin with metadata
dfx canister install me_coin \
  --network ic \
  --wasm canisters/me_coin/me_coin.wasm \
  --argument "(record {
    name = \"Me-Coin\";
    symbol = \"MEC\";
    decimals = 8;
    fee = 10000;
    total_supply = 1000000000000000;
    minting_account = opt record {
      owner = principal \"$GOVERNANCE_PRINCIPAL\";
      subaccount = null;
    };
    logo = opt \"https://marketearth.io/assets/me-coin.svg\";
  })"
```

### 2.2 Verify Me-Coin Metadata

```bash
# Query ICRC-1 metadata
dfx canister call me_coin icrc1_name --network ic
# Expected: ("Me-Coin")

dfx canister call me_coin icrc1_symbol --network ic
# Expected: ("MEC")

dfx canister call me_coin icrc1_decimals --network ic
# Expected: (8 : nat8)

dfx canister call me_coin icrc1_fee --network ic
# Expected: (10000 : nat)

dfx canister call me_coin icrc1_total_supply --network ic
# Expected: (1000000000000000 : nat)

dfx canister call me_coin icrc1_metadata --network ic
# Expected: Full metadata vector
```

### 2.3 Deploy We-Coin (ICRC-1 Token)

```bash
dfx canister install we_coin \
  --network ic \
  --wasm canisters/we_coin/we_coin.wasm \
  --argument "(record {
    name = \"We-Coin\";
    symbol = \"WEC\";
    decimals = 8;
    fee = 10000;
    total_supply = 500000000000000;
    minting_account = opt record {
      owner = principal \"$GOVERNANCE_PRINCIPAL\";
      subaccount = null;
    };
    logo = opt \"https://marketearth.io/assets/we-coin.svg\";
  })"
```

### 2.4 Verify We-Coin Metadata

```bash
dfx canister call we_coin icrc1_name --network ic
dfx canister call we_coin icrc1_symbol --network ic
dfx canister call we_coin icrc1_decimals --network ic
dfx canister call we_coin icrc1_fee --network ic
dfx canister call we_coin icrc1_total_supply --network ic
```

---

## Step 3: Initialize Liquidity Pair

### 3.1 Deploy Pair Canister (LP + Swaps)

```bash
# Deploy pair with token addresses and metadata
dfx canister install pair \
  --network ic \
  --wasm canisters/pair/pair.wasm \
  --argument "(record {
    token_a = principal \"$ME_COIN_ID\";
    token_b = principal \"$WE_COIN_ID\";
    fee_numerator = 30;  /* 0.3% */
    fee_denominator = 10000;
  })"
```

### 3.2 Verify Pair State

```bash
# Query initial reserves (should be 0)
dfx canister call pair get_reserves --network ic
# Expected: (0, 0)

# Query total LP supply (should be 0)
dfx canister call pair total_supply --network ic
# Expected: (0 : nat)

# Query pair ICRC-1 metadata
dfx canister call pair icrc1_name --network ic
# Expected: ("Market Earth LP")

dfx canister call pair icrc1_symbol --network ic
# Expected: ("ME-LP")

dfx canister call pair icrc1_decimals --network ic
# Expected: (8 : nat8)
```

### 3.3 Deploy Factory Canister

```bash
# Deploy factory with governance settings
dfx canister install amm_factory \
  --network ic \
  --wasm canisters/amm_factory/amm_factory.wasm \
  --argument "(record {
    governance_principal = principal \"$GOVERNANCE_PRINCIPAL\";
    paused = false;
    whitelist = vec {};
    pair_canister = principal \"$PAIR_ID\";
  })"
```

### 3.4 Verify Factory State

```bash
dfx canister call amm_factory get_config --network ic
# Expected: Config with governance_principal, paused=false

dfx canister call amm_factory list_pairs --network ic
# Expected: Empty vec initially
```

---

## Step 4: Initialize Liquidity

### 4.1 Transfer Initial Tokens to Pair

```bash
# Transfer Me-Coin to pair (100,000 MEC = 10_000_000_000_000 with 8 decimals)
dfx canister call me_coin transfer \
  --network ic \
  "(record {
    from_subaccount = null;
    to = record {
      owner = principal \"$PAIR_ID\";
      subaccount = null;
    };
    amount = 10000000000000;
    fee = opt 10000;
    memo = opt blob \"init-liquidity\";
    created_at_time = opt $(date +%s)$(printf '000000000');
  })"

# Transfer We-Coin to pair (50,000 WEC = 5_000_000_000_000 with 8 decimals)
dfx canister call we_coin transfer \
  --network ic \
  "(record {
    from_subaccount = null;
    to = record {
      owner = principal \"$PAIR_ID\";
      subaccount = null;
    };
    amount = 5000000000000;
    fee = opt 10000;
    memo = opt blob \"init-liquidity\";
    created_at_time = opt $(date +%s)$(printf '000000000');
  })"
```

### 4.2 Call add_liquidity on Pair

```bash
# Add liquidity to initialize the pool
dfx canister call pair add_liquidity \
  --network ic \
  "(10000000000000, 5000000000000)"
```

### 4.3 Verify Liquidity Added

```bash
# Check reserves
dfx canister call pair get_reserves --network ic
# Expected: (10000000000000, 5000000000000)

# Check LP supply
dfx canister call pair total_supply --network ic
# Expected: (7071067811865 : nat) -- sqrt(a * b)

# Check your LP balance
dfx canister call pair get_balance \
  --network ic \
  "(principal \"$GOVERNANCE_PRINCIPAL\")"
# Expected: (7071067811865 : nat)
```

---

## Step 5: Register on ICPSwap

### 5.1 Get Your Pool Metadata

```bash
cat > /tmp/icpswap_registration.json << EOF
{
  "token_a": "$ME_COIN_ID",
  "token_b": "$WE_COIN_ID",
  "pair_canister": "$PAIR_ID",
  "pool_name": "Market Earth (Me-Coin ↔ We-Coin)",
  "decimals_a": 8,
  "decimals_b": 8,
  "fee": 300,
  "logo_a": "https://marketearth.io/assets/me-coin.svg",
  "logo_b": "https://marketearth.io/assets/we-coin.svg",
  "verified": false
}
EOF

cat /tmp/icpswap_registration.json
```

### 5.2 Register Pool on ICPSwap

**Option A: Via ICPSwap Web UI**
1. Visit https://app.icpswap.com
2. Go to **Swap** → **Create New Pool** (or **Pools** → **Add Liquidity**)
3. Search for your token canister IDs:
   - Token A: `$ME_COIN_ID`
   - Token B: `$WE_COIN_ID`
4. If not found, toggle **"Show only listed tokens"** OFF
5. Set fee tier (0.3% = 3000 basis points)
6. Click **Create Pool**

**Option B: Via ICPSwap API**
```bash
# Submit your pool for indexing
curl -X POST https://api.icpswap.com/v1/pools \
  -H "Content-Type: application/json" \
  -d @/tmp/icpswap_registration.json
```

### 5.3 Verify Pool on ICPSwap

```bash
# Check ICPSwap indexer has picked up your pool
curl https://api.icpswap.com/v1/pools/$ME_COIN_ID/$WE_COIN_ID \
  | jq '.data'

# Expected output:
# {
#   "pool_id": "$PAIR_ID",
#   "token_a": "$ME_COIN_ID",
#   "token_b": "$WE_COIN_ID",
#   "reserve_a": "10000000000000",
#   "reserve_b": "5000000000000",
#   "fee": 300,
#   "liquidity": "7071067811865"
# }
```

---

## Step 6: Execute Test Swaps

### 6.1 Perform a Swap (Me-Coin → We-Coin)

```bash
# Swap 1,000 Me-Coin for We-Coin
# With 8 decimals: 1,000 = 100_000_000_000

dfx canister call pair swap \
  --network ic \
  "(principal \"$ME_COIN_ID\", 100000000000, 0)"

# Expected output: (amount_out : nat)
# This is the We-Coin amount received (minus 0.3% fee)
```

### 6.2 Verify Swap Execution

```bash
# Check new reserves
dfx canister call pair get_reserves --network ic
# Expected: Reserves changed, product k increased (due to fee accrual)

# Check your We-Coin balance
dfx canister call we_coin balance_of \
  --network ic \
  "(record {
    owner = principal \"$GOVERNANCE_PRINCIPAL\";
    subaccount = null;
  })"
```

### 6.3 Perform Reverse Swap (We-Coin → Me-Coin)

```bash
dfx canister call pair swap \
  --network ic \
  "(principal \"$WE_COIN_ID\", 50000000000, 0)"
```

---

## Step 7: Monitor & Verify

### 7.1 Query Pool Statistics

```bash
# Get current reserves and volume
dfx canister call pair get_reserves --network ic

# Get total LP supply
dfx canister call pair total_supply --network ic

# Get your LP balance
dfx canister call pair get_balance \
  --network ic \
  "(principal \"$GOVERNANCE_PRINCIPAL\")"
```

### 7.2 Check Token Balances

```bash
# Me-Coin balance
dfx canister call me_coin balance_of \
  --network ic \
  "(record {
    owner = principal \"$GOVERNANCE_PRINCIPAL\";
    subaccount = null;
  })"

# We-Coin balance
dfx canister call we_coin balance_of \
  --network ic \
  "(record {
    owner = principal \"$GOVERNANCE_PRINCIPAL\";
    subaccount = null;
  })"
```

### 7.3 Verify On-Chain State

```bash
# Check Me-Coin total supply
dfx canister call me_coin icrc1_total_supply --network ic

# Check We-Coin total supply
dfx canister call we_coin icrc1_total_supply --network ic

# Check pair LP total supply
dfx canister call pair icrc1_total_supply --network ic
```

---

## Summary of Deployed Canisters

Save these IDs for future reference:

```bash
echo "
=== Market Earth Canister IDs ===
ME_COIN_ID=$ME_COIN_ID
WE_COIN_ID=$WE_COIN_ID
PAIR_ID=$PAIR_ID
FACTORY_ID=$FACTORY_ID
GOVERNANCE_PRINCIPAL=$GOVERNANCE_PRINCIPAL
" >> ~/.market-earth-ids.txt

cat ~/.market-earth-ids.txt
```

---

## Troubleshooting

### Issue: "Canister does not exist"
```bash
# Ensure canister was created
dfx canister status amm_factory --network ic
```

### Issue: "Invalid Candid format"
```bash
# Verify your argument syntax
dfx canister install pair --network ic --dry-run
```

### Issue: "Unauthorized" on set_paused
```bash
# Ensure you're using the correct governance principal
dfx identity get-principal
```

### Issue: Swap fails with "Slippage exceeded"
```bash
# Increase min_amount_out tolerance
dfx canister call pair swap \
  --network ic \
  "(principal \"$ME_COIN_ID\", 100000000000, 1)"
# Instead of 0, use a small positive number
```

---

## Next Steps

1. ✅ Deploy Me-Coin & We-Coin (ICRC-1)
2. ✅ Deploy Pair & Factory canisters
3. ✅ Initialize liquidity pool
4. ✅ Register on ICPSwap
5. ✅ Perform test swaps
6. **TODO:** Connect Flask backend to canister IDs
7. **TODO:** Set up Cloudflare DNS routing
8. **TODO:** Configure GitHub Actions auto-deploy

---

**Questions?** Run any step and share the output!
