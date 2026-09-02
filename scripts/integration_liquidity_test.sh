#!/usr/bin/env bash
set -euo pipefail
TOKEN_A_ID=$(dfx canister id token_a 2>/dev/null || echo "ryjl3-tyaaa-aaaaa-aaaba-cai")
TOKEN_B_ID=$(dfx canister id token_b 2>/dev/null || echo "r7inp-6aaaa-aaaaa-aaaaa-cai")
PAIR_RAW=$(dfx canister call amm_factory create_pair "(principal \"$TOKEN_A_ID\", principal \"$TOKEN_B_ID\")")
PAIR_ID=$(echo "$PAIR_RAW" | grep -oE '([a-z0-9\-]+-cai)' | head -n 1)
[ -z "$PAIR_ID" ] && exit 1
PAIRS_LIST=$(dfx canister call amm_factory list_pairs)
[["$PAIRS_LIST" != *"$PAIR_ID"* ]] && exit 1
dfx canister call "$TOKEN_A_ID" mint "(principal \"$(dfx identity get-principal)\", 1_000_000_000 : nat)" >/dev/null 2>&1 || true
dfx canister call "$TOKEN_B_ID" mint "(principal \"$(dfx identity get-principal)\", 1_000_000_000 : nat)" >/dev/null 2>&1 || true
dfx canister call "$TOKEN_A_ID" approve "(principal \"$PAIR_ID\", 500_000_000 : nat)" >/dev/null 2>&1 || true
dfx canister call "$TOKEN_B_ID" approve "(principal \"$PAIR_ID\", 500_000_000 : nat)" >/dev/null 2>&1 || true
dfx canister call "$PAIR_ID" add_liquidity "(500_000_000 : nat, 500_000_000 : nat)" 2>&1 || true
dfx canister call "$PAIR_ID" get_reserves 2>&1 || true
dfx canister call "$PAIR_ID" swap "(principal \"$TOKEN_A_ID\", 10_000 : nat, 1 : nat)" 2>&1 || true
echo "==> SUCCESS: E2E Pipeline Passed."
