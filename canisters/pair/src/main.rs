use candid::{CandidType, Deserialize, Nat, Principal};
use ic_cdk::api::call::RejectionCode;
use ic_cdk::{query, update};
use std::cell::RefCell;
use std::collections::BTreeMap;

const FEE_BPS: u128 = 30;
const FEE_DENOMINATOR: u128 = 10_000;

#[derive(CandidType, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Account {
    pub owner: Principal,
    pub subaccount: Option<Vec<u8>>,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct TransferArg {
    pub from_subaccount: Option<Vec<u8>>,
    pub to: Account,
    pub amount: Nat,
    pub fee: Option<Nat>,
    pub memo: Option<Vec<u8>>,
    pub created_at_time: Option<u64>,
}

#[derive(CandidType, Deserialize, Debug)]
pub enum TransferError {
    BadFee { expected_fee: Nat },
    BadBurn { min_burn_amount: Nat },
    InsufficientFunds { balance: Nat },
    TooOld,
    CreatedInFuture { ledger_time: u64 },
    TemporarilyUnavailable,
    Duplicate { duplicate_of: Nat },
    GenericError { error_code: Nat, message: String },
}

type TransferResult = Result<Nat, TransferError>;

#[derive(CandidType, Deserialize, Clone)]
pub struct TransferFromArgs {
    pub spender_subaccount: Option<Vec<u8>>,
    pub from: Account,
    pub to: Account,
    pub amount: Nat,
    pub fee: Option<Nat>,
    pub memo: Option<Vec<u8>>,
    pub created_at_time: Option<u64>,
}

#[derive(CandidType, Deserialize, Debug)]
pub enum TransferFromError {
    BadFee { expected_fee: Nat },
    BadBurn { min_burn_amount: Nat },
    InsufficientFunds { balance: Nat },
    InsufficientAllowance { allowance: Nat },
    TooOld,
    CreatedInFuture { ledger_time: u64 },
    Duplicate { duplicate_of: Nat },
    TemporarilyUnavailable,
    GenericError { error_code: Nat, message: String },
}

type TransferFromResult = Result<Nat, TransferFromError>;

#[derive(CandidType, Deserialize, Clone)]
pub struct PairState {
    pub token_a: Principal,
    pub token_b: Principal,
    pub reserve_a: u128,
    pub reserve_b: u128,
    pub total_supply: u128,
    pub balances: BTreeMap<Principal, u128>,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct PairInfo {
    pub token_a: Principal,
    pub token_b: Principal,
    pub reserve_a: u128,
    pub reserve_b: u128,
    pub total_supply: u128,
    pub fee_basis_points: u16,
}

impl Default for PairState {
    fn default() -> Self {
        Self {
            token_a: Principal::anonymous(),
            token_b: Principal::anonymous(),
            reserve_a: 0,
            reserve_b: 0,
            total_supply: 0,
            balances: BTreeMap::new(),
        }
    }
}

thread_local! {
    static PAIR_STATE: RefCell<PairState> = RefCell::new(PairState::default());
    static OPERATION_IN_PROGRESS: RefCell<bool> = const { RefCell::new(false) };
}

struct OperationGuard;

impl OperationGuard {
    fn begin() -> Result<Self, String> {
        OPERATION_IN_PROGRESS.with(|flag| {
            let mut in_progress = flag.borrow_mut();
            if *in_progress {
                return Err("Another operation is currently in progress".to_string());
            }
            *in_progress = true;
            Ok(Self)
        })
    }
}

impl Drop for OperationGuard {
    fn drop(&mut self) {
        OPERATION_IN_PROGRESS.with(|flag| {
            *flag.borrow_mut() = false;
        });
    }
}

fn canonical_tokens(
    token_a: Principal,
    token_b: Principal,
) -> Result<(Principal, Principal), String> {
    if token_a == token_b {
        return Err("Tokens must be different".to_string());
    }
    if token_a < token_b {
        Ok((token_a, token_b))
    } else {
        Ok((token_b, token_a))
    }
}

fn nat_to_u128(value: &Nat) -> Result<u128, String> {
    // candid::Nat Display inserts `_` digit separators; strip before parsing.
    value
        .to_string()
        .replace('_', "")
        .parse::<u128>()
        .map_err(|_| "Amount exceeds u128 range".to_string())
}

fn integer_sqrt(value: u128) -> u128 {
    if value <= 1 {
        return value;
    }
    let mut x0 = value;
    let mut x1 = (x0 + value / x0) / 2;
    while x1 < x0 {
        x0 = x1;
        x1 = (x0 + value / x0) / 2;
    }
    x0
}

fn compute_lp_tokens(
    reserve_a: u128,
    reserve_b: u128,
    total_supply: u128,
    amount_a: u128,
    amount_b: u128,
) -> Result<u128, String> {
    if total_supply == 0 {
        let product = amount_a
            .checked_mul(amount_b)
            .ok_or_else(|| "Liquidity product overflow".to_string())?;
        return Ok(integer_sqrt(product));
    }

    if reserve_a == 0 || reserve_b == 0 {
        return Err("Invalid reserves for non-empty pool".to_string());
    }

    let liquidity_a = amount_a
        .checked_mul(total_supply)
        .ok_or_else(|| "Liquidity calculation overflow".to_string())?
        / reserve_a;
    let liquidity_b = amount_b
        .checked_mul(total_supply)
        .ok_or_else(|| "Liquidity calculation overflow".to_string())?
        / reserve_b;
    Ok(std::cmp::min(liquidity_a, liquidity_b))
}

fn quote_swap_internal(
    reserve_in: u128,
    reserve_out: u128,
    amount_in: u128,
) -> Result<u128, String> {
    if reserve_in == 0 || reserve_out == 0 {
        return Err("Pool has no liquidity".to_string());
    }

    let fee_multiplier = FEE_DENOMINATOR
        .checked_sub(FEE_BPS)
        .ok_or_else(|| "Invalid fee configuration".to_string())?;
    let amount_in_with_fee = amount_in
        .checked_mul(fee_multiplier)
        .ok_or_else(|| "Amount overflow".to_string())?;
    let numerator = amount_in_with_fee
        .checked_mul(reserve_out)
        .ok_or_else(|| "Swap numerator overflow".to_string())?;
    let denominator = reserve_in
        .checked_mul(FEE_DENOMINATOR)
        .and_then(|base| base.checked_add(amount_in_with_fee))
        .ok_or_else(|| "Swap denominator overflow".to_string())?;

    if denominator == 0 {
        return Err("Invalid swap denominator".to_string());
    }

    let amount_out = numerator / denominator;
    if amount_out == 0 {
        return Err("Amount out is zero".to_string());
    }
    if amount_out >= reserve_out {
        return Err("Insufficient liquidity".to_string());
    }
    Ok(amount_out)
}

fn pair_account() -> Account {
    Account {
        owner: ic_cdk::api::id(),
        subaccount: None,
    }
}

fn caller_account() -> Account {
    Account {
        owner: ic_cdk::api::caller(),
        subaccount: None,
    }
}

fn format_call_error(context: &str, code: RejectionCode, message: String) -> String {
    format!("{context} rejected ({code:?}): {message}")
}

async fn token_transfer_from(token: Principal, from: Account, amount: u128) -> Result<(), String> {
    let args = TransferFromArgs {
        spender_subaccount: None,
        from,
        to: pair_account(),
        amount: Nat::from(amount),
        fee: None,
        memo: None,
        created_at_time: None,
    };

    let (result,): (TransferFromResult,) = ic_cdk::call(token, "icrc2_transfer_from", (args,))
        .await
        .map_err(|(code, message)| format_call_error("icrc2_transfer_from", code, message))?;

    match result {
        Ok(_) => Ok(()),
        Err(err) => Err(format!("icrc2_transfer_from token {token} failed: {err:?}")),
    }
}

async fn token_transfer(token: Principal, to: Account, amount: u128) -> Result<(), String> {
    let args = TransferArg {
        from_subaccount: None,
        to,
        amount: Nat::from(amount),
        fee: None,
        memo: None,
        created_at_time: None,
    };

    let (result,): (TransferResult,) = ic_cdk::call(token, "icrc1_transfer", (args,))
        .await
        .map_err(|(code, message)| format_call_error("icrc1_transfer", code, message))?;

    match result {
        Ok(_) => Ok(()),
        Err(err) => Err(format!("icrc1_transfer token {token} failed: {err:?}")),
    }
}

#[ic_cdk::init]
fn init(token_a: Principal, token_b: Principal) {
    let (token_a, token_b) = match canonical_tokens(token_a, token_b) {
        Ok(tokens) => tokens,
        Err(message) => ic_cdk::trap(&message),
    };

    PAIR_STATE.with(|state| {
        let mut s = state.borrow_mut();
        s.token_a = token_a;
        s.token_b = token_b;
    });
}

#[update]
async fn add_liquidity(amount_a: u128, amount_b: u128) -> Result<u128, String> {
    let _guard = OperationGuard::begin()?;

    if amount_a == 0 || amount_b == 0 {
        return Err("Amounts must be greater than zero".to_string());
    }

    let caller = caller_account();
    let (token_a, token_b, lp_tokens) = PAIR_STATE.with(|state| {
        let s = state.borrow();
        let lp_tokens =
            compute_lp_tokens(s.reserve_a, s.reserve_b, s.total_supply, amount_a, amount_b)?;
        if lp_tokens == 0 {
            return Err("Insufficient liquidity minted".to_string());
        }
        Ok((s.token_a, s.token_b, lp_tokens))
    })?;

    token_transfer_from(token_a, caller.clone(), amount_a).await?;

    if let Err(err) = token_transfer_from(token_b, caller.clone(), amount_b).await {
        let refund_err = token_transfer(token_a, caller.clone(), amount_a)
            .await
            .err();
        return Err(match refund_err {
            Some(refund) => format!(
                "Second transfer failed ({err}); refund of first transfer also failed ({refund})"
            ),
            None => format!("Second transfer failed ({err}); refunded first transfer"),
        });
    }

    PAIR_STATE.with(|state| {
        let mut s = state.borrow_mut();
        s.reserve_a = s
            .reserve_a
            .checked_add(amount_a)
            .ok_or_else(|| "reserve_a overflow".to_string())?;
        s.reserve_b = s
            .reserve_b
            .checked_add(amount_b)
            .ok_or_else(|| "reserve_b overflow".to_string())?;
        s.total_supply = s
            .total_supply
            .checked_add(lp_tokens)
            .ok_or_else(|| "total_supply overflow".to_string())?;

        let caller_lp = s.balances.get(&caller.owner).copied().unwrap_or(0);
        let new_lp = caller_lp
            .checked_add(lp_tokens)
            .ok_or_else(|| "LP balance overflow".to_string())?;
        s.balances.insert(caller.owner, new_lp);
        Ok(lp_tokens)
    })
}

#[update]
async fn remove_liquidity(lp_amount: u128) -> Result<(u128, u128), String> {
    let _guard = OperationGuard::begin()?;

    if lp_amount == 0 {
        return Err("LP amount must be greater than zero".to_string());
    }

    let caller = caller_account();
    let (token_a, token_b, amount_a, amount_b) = PAIR_STATE.with(|state| {
        let mut s = state.borrow_mut();
        let balance = s.balances.get(&caller.owner).copied().unwrap_or(0);
        if balance < lp_amount {
            return Err("Insufficient LP tokens".to_string());
        }
        if s.total_supply == 0 {
            return Err("Pool has no LP supply".to_string());
        }

        let amount_a = lp_amount
            .checked_mul(s.reserve_a)
            .ok_or_else(|| "amount_a overflow".to_string())?
            / s.total_supply;
        let amount_b = lp_amount
            .checked_mul(s.reserve_b)
            .ok_or_else(|| "amount_b overflow".to_string())?
            / s.total_supply;

        if amount_a == 0 || amount_b == 0 {
            return Err("Withdrawal amounts are zero".to_string());
        }

        s.balances.insert(caller.owner, balance - lp_amount);
        s.total_supply -= lp_amount;
        s.reserve_a = s
            .reserve_a
            .checked_sub(amount_a)
            .ok_or_else(|| "reserve_a underflow".to_string())?;
        s.reserve_b = s
            .reserve_b
            .checked_sub(amount_b)
            .ok_or_else(|| "reserve_b underflow".to_string())?;

        Ok((s.token_a, s.token_b, amount_a, amount_b))
    })?;

    if let Err(err) = token_transfer(token_a, caller.clone(), amount_a).await {
        PAIR_STATE.with(|state| {
            let mut s = state.borrow_mut();
            let current_lp = s.balances.get(&caller.owner).copied().unwrap_or(0);
            s.balances.insert(caller.owner, current_lp + lp_amount);
            s.total_supply += lp_amount;
            s.reserve_a += amount_a;
            s.reserve_b += amount_b;
        });
        return Err(format!("Failed to transfer token_a: {err}"));
    }

    if let Err(err) = token_transfer(token_b, caller.clone(), amount_b).await {
        PAIR_STATE.with(|state| {
            let mut s = state.borrow_mut();
            s.reserve_b += amount_b;
        });
        return Err(format!(
            "Failed to transfer token_b after token_a was sent. \
             State kept consistent and LP burn remains applied: {err}"
        ));
    }

    Ok((amount_a, amount_b))
}

#[update]
async fn swap(token_in: Principal, amount_in: u128, min_amount_out: u128) -> Result<u128, String> {
    let _guard = OperationGuard::begin()?;

    if amount_in == 0 {
        return Err("Amount in must be greater than zero".to_string());
    }

    let caller = caller_account();
    let (token_a, token_b, reserve_a, reserve_b) = PAIR_STATE.with(|state| {
        let s = state.borrow();
        (s.token_a, s.token_b, s.reserve_a, s.reserve_b)
    });

    let (token_out, amount_out, input_is_a) = if token_in == token_a {
        (
            token_b,
            quote_swap_internal(reserve_a, reserve_b, amount_in)?,
            true,
        )
    } else if token_in == token_b {
        (
            token_a,
            quote_swap_internal(reserve_b, reserve_a, amount_in)?,
            false,
        )
    } else {
        return Err("Invalid token".to_string());
    };

    if amount_out < min_amount_out {
        return Err("Slippage exceeded".to_string());
    }

    token_transfer_from(token_in, caller.clone(), amount_in).await?;

    PAIR_STATE.with(|state| {
        let mut s = state.borrow_mut();
        if input_is_a {
            s.reserve_a = s
                .reserve_a
                .checked_add(amount_in)
                .ok_or_else(|| "reserve_a overflow".to_string())?;
        } else {
            s.reserve_b = s
                .reserve_b
                .checked_add(amount_in)
                .ok_or_else(|| "reserve_b overflow".to_string())?;
        }
        Ok::<(), String>(())
    })?;

    if let Err(err) = token_transfer(token_out, caller.clone(), amount_out).await {
        let refund_err = token_transfer(token_in, caller.clone(), amount_in)
            .await
            .err();
        match refund_err {
            Some(refund) => {
                return Err(format!(
                    "Swap output transfer failed ({err}); input refund also failed ({refund})"
                ));
            }
            None => {
                PAIR_STATE.with(|state| {
                    let mut s = state.borrow_mut();
                    if input_is_a {
                        s.reserve_a -= amount_in;
                    } else {
                        s.reserve_b -= amount_in;
                    }
                });
                return Err(format!(
                    "Swap output transfer failed ({err}); input amount was refunded"
                ));
            }
        }
    }

    PAIR_STATE.with(|state| {
        let mut s = state.borrow_mut();
        if input_is_a {
            s.reserve_b = s
                .reserve_b
                .checked_sub(amount_out)
                .ok_or_else(|| "reserve_b underflow".to_string())?;
        } else {
            s.reserve_a = s
                .reserve_a
                .checked_sub(amount_out)
                .ok_or_else(|| "reserve_a underflow".to_string())?;
        }
        Ok::<(), String>(())
    })?;

    Ok(amount_out)
}

#[query]
fn quote_swap(token_in: Principal, amount_in: Nat) -> Result<Nat, String> {
    let amount_in = nat_to_u128(&amount_in)?;
    PAIR_STATE.with(|state| {
        let s = state.borrow();
        if token_in == s.token_a {
            quote_swap_internal(s.reserve_a, s.reserve_b, amount_in).map(Nat::from)
        } else if token_in == s.token_b {
            quote_swap_internal(s.reserve_b, s.reserve_a, amount_in).map(Nat::from)
        } else {
            Err("Invalid token".to_string())
        }
    })
}

#[query]
fn get_pair_info() -> PairInfo {
    PAIR_STATE.with(|state| {
        let s = state.borrow();
        PairInfo {
            token_a: s.token_a,
            token_b: s.token_b,
            reserve_a: s.reserve_a,
            reserve_b: s.reserve_b,
            total_supply: s.total_supply,
            fee_basis_points: FEE_BPS as u16,
        }
    })
}

#[query]
fn get_reserves() -> (u128, u128) {
    PAIR_STATE.with(|state| {
        let s = state.borrow();
        (s.reserve_a, s.reserve_b)
    })
}

#[query]
fn get_balance(account: Principal) -> u128 {
    PAIR_STATE.with(|state| {
        let s = state.borrow();
        s.balances.get(&account).copied().unwrap_or(0)
    })
}

#[query]
fn total_supply() -> u128 {
    PAIR_STATE.with(|state| state.borrow().total_supply)
}

ic_cdk::export_candid!();

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn pair_state_default_is_empty() {
        let state = PairState::default();

        assert_eq!(state.token_a, Principal::anonymous());
        assert_eq!(state.token_b, Principal::anonymous());
        assert_eq!(state.reserve_a, 0);
        assert_eq!(state.reserve_b, 0);
        assert_eq!(state.total_supply, 0);
        assert!(state.balances.is_empty());
    }

    #[test]
    fn quote_swap_applies_fee() {
        let amount_out = quote_swap_internal(1_000_000, 500_000, 10_000).unwrap();
        assert_eq!(amount_out, 4_935);
    }

    #[test]
    fn nat_to_u128_handles_underscore_separators() {
        let value = Nat::from(1_000_000_000_000_000u128);
        assert_eq!(nat_to_u128(&value).unwrap(), 1_000_000_000_000_000u128);
    }

    #[test]
    fn candid_interface_matches_checked_in_did() {
        let generated_candid = __export_service();
        let did_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("pair.did");
        let checked_in_candid =
            std::fs::read_to_string(&did_path).expect("failed to read checked-in pair.did");

        assert_eq!(
            generated_candid.trim_end(),
            checked_in_candid.trim_end(),
            "pair.did is out of date"
        );
    }
}
