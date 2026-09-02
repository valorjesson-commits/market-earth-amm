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
dfx canister install --all --mode reinstall

# Get canister IDs
echo ""
echo "=== Deployment Complete ==="
dfx canister id amm 2>/dev/null || echo "AMM canister deployed"

echo "Local deployment finished successfully!"