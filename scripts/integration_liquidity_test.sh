#!/usr/bin/env bash
set -euo pipefail

export PATH="$HOME/.local/share/dfx/bin:$PATH"
DFX="${DFX_BIN:-dfx}"

if ! command -v "$DFX" >/dev/null 2>&1; then
  echo "dfx is not installed or not on PATH"
  exit 1
fi

# dfx prints large nats with `_` separators; strip them before parsing.
strip_underscores() {
  tr -d '_'
}

extract_first_number() {
  strip_underscores | grep -oE '[0-9]+' | head -n1
}

extract_ok_number() {
  sed -n 's/.*Ok = \([0-9][0-9_]*\).*/\1/p' | tr -d '_' | head -n1
}

echo "=== Market Earth AMM Integration & Liquidity Tests ==="

GOV_PRINCIPAL="$("$DFX" identity get-principal)"
TOKEN_A_ID="$("$DFX" canister id token_a)"
TOKEN_B_ID="$("$DFX" canister id token_b)"
PAIR_ID="$("$DFX" canister id pair)"
FACTORY_ID="$("$DFX" canister id amm_factory)"

echo "Factory: $FACTORY_ID"
echo "Pair:    $PAIR_ID"

echo ""
echo "Test 1: Factory and pair registration"
"$DFX" canister call amm_factory get_config >/dev/null
PAIR_LIST_OUTPUT="$("$DFX" canister call amm_factory list_pairs)"
echo "$PAIR_LIST_OUTPUT" | grep -q "$PAIR_ID"
echo "✓ Factory config is readable and pair is registered"

APPROVE_AMOUNT=500000000000
LIQ_A=120000000000
LIQ_B=60000000000
SWAP_IN=1000000000

echo ""
echo "Test 2: Approve pair canister as spender"
"$DFX" canister call token_a icrc2_approve "(record {
  from_subaccount = null;
  spender = record { owner = principal \"$PAIR_ID\"; subaccount = null };
  amount = $APPROVE_AMOUNT : nat;
  expected_allowance = null;
  expires_at = null;
  fee = null;
  memo = null;
  created_at_time = null;
})" >/dev/null

"$DFX" canister call token_b icrc2_approve "(record {
  from_subaccount = null;
  spender = record { owner = principal \"$PAIR_ID\"; subaccount = null };
  amount = $APPROVE_AMOUNT : nat;
  expected_allowance = null;
  expires_at = null;
  fee = null;
  memo = null;
  created_at_time = null;
})" >/dev/null
echo "✓ Allowances granted"

echo ""
echo "Test 3: Add liquidity"
ADD_OUTPUT="$("$DFX" canister call pair add_liquidity "($LIQ_A : nat, $LIQ_B : nat)")"
echo "$ADD_OUTPUT" | grep -q "variant { Ok"
RESERVES_AFTER_ADD="$("$DFX" canister call pair get_reserves)"
echo "$RESERVES_AFTER_ADD" | strip_underscores | grep -q "$LIQ_A"
echo "$RESERVES_AFTER_ADD" | strip_underscores | grep -q "$LIQ_B"
echo "✓ Liquidity added and reserves updated"

echo ""
echo "Test 4: Quote and execute swap"
QUOTE_OUTPUT="$("$DFX" canister call pair quote_swap "(principal \"$TOKEN_A_ID\", $SWAP_IN : nat)")"
QUOTE_AMOUNT="$(echo "$QUOTE_OUTPUT" | extract_ok_number)"
if [[ -z "$QUOTE_AMOUNT" || "$QUOTE_AMOUNT" -le 0 ]]; then
  echo "✗ Invalid quote output: $QUOTE_OUTPUT"
  exit 1
fi

SWAP_OUTPUT="$("$DFX" canister call pair swap "(principal \"$TOKEN_A_ID\", $SWAP_IN : nat, 1 : nat)")"
echo "$SWAP_OUTPUT" | grep -q "variant { Ok"
echo "✓ Swap executed with positive output"

echo ""
echo "Test 5: Remove partial liquidity"
LP_BALANCE_OUTPUT="$("$DFX" canister call pair get_balance "(principal \"$GOV_PRINCIPAL\")")"
LP_BALANCE="$(echo "$LP_BALANCE_OUTPUT" | extract_first_number)"
WITHDRAW_LP=$((LP_BALANCE / 2))
if [[ "$WITHDRAW_LP" -eq 0 ]]; then
  WITHDRAW_LP=1
fi
REMOVE_OUTPUT="$("$DFX" canister call pair remove_liquidity "($WITHDRAW_LP : nat)")"
echo "$REMOVE_OUTPUT" | grep -q "variant { Ok"
echo "✓ Liquidity removal succeeded"

echo ""
echo "=== All Integration Tests Passed ==="