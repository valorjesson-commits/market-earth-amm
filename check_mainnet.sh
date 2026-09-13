#!/usr/bin/env bash
set -e
echo "=== 1. CANISTER_ID ==="; FACTORY_ID=$(dfx canister --network ic id amm_factory 2>/dev/null || echo "Not found"); echo "$FACTORY_ID"
if [ "$FACTORY_ID" != "Not found" ]; then
    echo "=== 2. GET_CONFIG ==="; dfx canister --network ic call amm_factory get_config || true
    echo "=== 3. STATUS ==="; dfx canister --network ic status "$FACTORY_ID" || true
    echo "=== 4. LIST_PAIRS ==="; dfx canister --network ic call amm_factory list_pairs || true
    echo "=== 5. PAIR_CANISTER ==="; PAIR_ID=$(dfx canister --network ic id pair 2>/dev/null || echo "Not found"); echo "$PAIR_ID"
    if [ "$PAIR_ID" != "Not found" ]; then
        echo "=== 6. PAIR_STATE ==="; dfx canister --network ic call pair get_reserves || true
        echo "=== 7. PAIR_SNAPSHOT ==="; dfx canister --network ic call pair get_snapshot || true
        echo "=== 8. PAIR_SUPPLY ==="; dfx canister --network ic call pair total_supply || true
    fi
fi
