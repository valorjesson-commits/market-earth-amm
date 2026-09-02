#!/usr/bin/env bash
set -e
echo "=== 1. CANISTER_ID ==="; FACTORY_ID=$(dfx canister --network ic id amm_factory 2>/dev/null || echo "Not found"); echo "$FACTORY_ID"
if [ "$FACTORY_ID" != "Not found" ]; then
    echo "=== 2. GET_CONFIG ==="; dfx canister --network ic call amm_factory get_config || true
    echo "=== 3. STATUS ==="; dfx canister --network ic status "$FACTORY_ID" || true
    echo "=== 4. LIST_PAIRS ==="; dfx canister --network ic call amm_factory list_pairs || true
fi
