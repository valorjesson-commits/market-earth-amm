#!/bin/bash
set -e

export PATH="$HOME/.local/share/dfx/bin:$PATH"

echo "=== Market Earth AMM Local Deployment ==="
GOV_PRINCIPAL="${GOV_PRINCIPAL:-$(dfx identity get-principal)}"
TOKEN_A_PRINCIPAL="${TOKEN_A_PRINCIPAL:-2vxsx-fae}"
TOKEN_B_PRINCIPAL="${TOKEN_B_PRINCIPAL:-aaaaa-aa}"

echo "GOV_PRINCIPAL: $GOV_PRINCIPAL"
echo "TOKEN_A_PRINCIPAL: $TOKEN_A_PRINCIPAL"
echo "TOKEN_B_PRINCIPAL: $TOKEN_B_PRINCIPAL"

# Start the local replica
echo "Starting DFX replica..."
dfx start --background

# Wait for replica to be ready
echo "Waiting for replica to be ready..."
sleep 5

# Create canisters
echo "Creating canisters..."
dfx canister create --all || true

# Build the project
echo "Building canisters..."
dfx build

# Deploy canisters
echo "Deploying canisters..."
dfx canister install pair --mode reinstall --yes
dfx canister install amm_factory --mode reinstall --yes --argument "(record {
  governance_principal = principal \"$GOV_PRINCIPAL\";
  paused = false;
  whitelist = vec {};
  pair_canister = principal \"$(dfx canister id pair)\";
})"

echo "Configuring the initial pair..."
dfx canister call amm_factory create_pair "(principal \"$TOKEN_A_PRINCIPAL\", principal \"$TOKEN_B_PRINCIPAL\")"

# Get canister IDs
echo ""
echo "=== Deployment Complete ==="
echo "AMM Factory: $(dfx canister id amm_factory)"
echo "Pair: $(dfx canister id pair)"
echo "Initial token pair: $TOKEN_A_PRINCIPAL / $TOKEN_B_PRINCIPAL"

echo ""
echo "Local deployment finished successfully!"
