#!/usr/bin/env bash
set -euo pipefail

export PATH="$HOME/.local/share/dfx/bin:$PATH"
DFX="${DFX_BIN:-dfx}"

if ! command -v "$DFX" >/dev/null 2>&1; then
  echo "dfx is not installed or not on PATH"
  exit 1
fi

GOV_PRINCIPAL="${GOV_PRINCIPAL:-$("$DFX" identity get-principal)}"

cleanup_local_network() {
  "$DFX" stop >/dev/null 2>&1 || true

  local stale_pids
  stale_pids="$(
    ss -ltnp 2>/dev/null \
      | sed -n 's/.*127.0.0.1:4943.*pid=\([0-9][0-9]*\).*/\1/p' \
      | sort -u
  )"

  if [[ -n "$stale_pids" ]]; then
    echo "Stopping stale local network processes on 127.0.0.1:4943: $stale_pids"
    kill $stale_pids >/dev/null 2>&1 || true
    sleep 1
  fi
}

echo "=== Market Earth AMM Local Deployment ==="
"$DFX" --version
echo "Governance principal: $GOV_PRINCIPAL"

cleanup_local_network
echo "Starting local network..."
"$DFX" start --background --clean

echo "Creating canisters..."
"$DFX" canister create --all

echo "Building canisters..."
"$DFX" build

INITIAL_SUPPLY=1000000000000000

echo "Deploying token canisters..."
"$DFX" deploy token_a --mode reinstall --yes --argument "(record {
  name = \"Me Coin\";
  symbol = \"MEC\";
  decimals = 8 : nat8;
  initial_balances = vec {
    record {
      account = record { owner = principal \"$GOV_PRINCIPAL\"; subaccount = null };
      amount = $INITIAL_SUPPLY : nat;
    }
  };
})"

"$DFX" deploy token_b --mode reinstall --yes --argument "(record {
  name = \"We Coin\";
  symbol = \"WEC\";
  decimals = 8 : nat8;
  initial_balances = vec {
    record {
      account = record { owner = principal \"$GOV_PRINCIPAL\"; subaccount = null };
      amount = $INITIAL_SUPPLY : nat;
    }
  };
})"

TOKEN_A_ID="$("$DFX" canister id token_a)"
TOKEN_B_ID="$("$DFX" canister id token_b)"

echo "Deploying pair canister..."
"$DFX" deploy pair --mode reinstall --yes \
  --argument "(principal \"$TOKEN_A_ID\", principal \"$TOKEN_B_ID\")"
PAIR_ID="$("$DFX" canister id pair)"

echo "Deploying factory canister..."
"$DFX" deploy amm_factory --mode reinstall --yes --argument "(record {
  governance_principal = principal \"$GOV_PRINCIPAL\";
  paused = false;
  whitelist = vec {};
})"

echo "Registering pair in factory..."
"$DFX" canister call amm_factory create_pair \
  "(principal \"$TOKEN_A_ID\", principal \"$TOKEN_B_ID\", principal \"$PAIR_ID\")"

echo ""
echo "=== Deployment Complete ==="
echo "token_a:     $TOKEN_A_ID"
echo "token_b:     $TOKEN_B_ID"
echo "pair:        $PAIR_ID"
echo "amm_factory: $("$DFX" canister id amm_factory)"

echo ""
echo "Local deployment finished successfully."
