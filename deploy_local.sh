#!/bin/bash
set -e

export PATH="$HOME/.local/share/dfx/bin:$PATH"

echo "=== Market Earth AMM Local Deployment ==="
echo "GOV_PRINCIPAL: $GOV_PRINCIPAL"

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
dfx canister install amm_factory --mode reinstall
dfx canister install token_a --mode reinstall
dfx canister install token_b --mode reinstall

# Get canister IDs
echo ""
echo "=== Deployment Complete ==="
echo "AMM Factory: $(dfx canister id amm_factory)"
echo "Token A: $(dfx canister id token_a)"
echo "Token B: $(dfx canister id token_b)"

echo ""
echo "Local deployment finished successfully!"
