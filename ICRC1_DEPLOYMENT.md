# ICRC Token Integration Guide

This AMM pair canister interacts with token canisters through:

- `icrc1_transfer` (pair sends tokens out)
- `icrc2_transfer_from` (pair pulls tokens in after user approval)

## Required token capabilities

Before integrating a token pair, verify both token canisters support:

1. `icrc1_transfer`
2. `icrc2_approve`
3. `icrc2_transfer_from`

## Example flow

### 1) Approve pair as spender

Call on each token canister:

```candid
icrc2_approve({
  from_subaccount = null;
  spender = { owner = principal "<PAIR_CANISTER>"; subaccount = null };
  amount = <ALLOWANCE>;
  expected_allowance = null;
  expires_at = null;
  fee = null;
  memo = null;
  created_at_time = null;
})
```

### 2) Add liquidity

```candid
add_liquidity(<amount_token_a>, <amount_token_b>)
```

### 3) Quote + swap

```candid
quote_swap(principal "<TOKEN_A>", <amount_in>)
swap(principal "<TOKEN_A>", <amount_in>, <min_amount_out>)
```

### 4) Remove liquidity

```candid
remove_liquidity(<lp_amount>)
```

## Factory registration

After deploying a pair canister, register it:

```candid
create_pair(
  principal "<TOKEN_A>",
  principal "<TOKEN_B>",
  principal "<PAIR_CANISTER>"
)
```
