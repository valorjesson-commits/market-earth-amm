use candid::{CandidType, Deserialize, Principal};
use ic_cdk::{query, update};
use std::cell::RefCell;

#[derive(CandidType, Deserialize, Clone, Default)]
pub struct PairState {
    pub token_a: Principal,
    pub token_b: Principal,
    pub reserve_a: u128,
    pub reserve_b: u128,
    pub total_supply: u128,
    pub balances: std::collections::HashMap<Principal, u128>,
}

thread_local! {
    static PAIR_STATE: RefCell<PairState> = RefCell::new(PairState::default());
}

#[ic_cdk::init]
fn init(token_a: Principal, token_b: Principal) {
    PAIR_STATE.with(|state| {
        let mut s = state.borrow_mut();
        s.token_a = token_a;
        s.token_b = token_b;
    });
}

#[update]
fn add_liquidity(amount_a: u128, amount_b: u128) -> Result<u128, String> {
    if amount_a == 0 || amount_b == 0 {
        return Err("Amounts must be greater than zero".to_string());
    }

    let caller = ic_cdk::api::caller();

    PAIR_STATE.with(|state| {
        let mut s = state.borrow_mut();

        let lp_tokens = if s.total_supply == 0 {
            (amount_a * amount_b).isqrt()
        } else {
            let liquidity_a = (amount_a * s.total_supply) / s.reserve_a;
            let liquidity_b = (amount_b * s.total_supply) / s.reserve_b;
            std::cmp::min(liquidity_a, liquidity_b)
        };

        if lp_tokens == 0 {
            return Err("Insufficient liquidity minted".to_string());
        }

        s.reserve_a += amount_a;
        s.reserve_b += amount_b;
        s.total_supply += lp_tokens;
        *s.balances.entry(caller).or_insert(0) += lp_tokens;

        Ok(lp_tokens)
    })
}

#[update]
fn remove_liquidity(lp_amount: u128) -> Result<(u128, u128), String> {
    if lp_amount == 0 {
        return Err("LP amount must be greater than zero".to_string());
    }

    let caller = ic_cdk::api::caller();

    PAIR_STATE.with(|state| {
        let mut s = state.borrow_mut();

        let balance = s.balances.get(&caller).copied().unwrap_or(0);
        if balance < lp_amount {
            return Err("Insufficient LP tokens".to_string());
        }

        let amount_a = (lp_amount * s.reserve_a) / s.total_supply;
        let amount_b = (lp_amount * s.reserve_b) / s.total_supply;

        s.balances.insert(caller, balance - lp_amount);
        s.reserve_a -= amount_a;
        s.reserve_b -= amount_b;
        s.total_supply -= lp_amount;

        Ok((amount_a, amount_b))
    })
}

#[update]
fn swap(token_in: Principal, amount_in: u128, min_amount_out: u128) -> Result<u128, String> {
    if amount_in == 0 {
        return Err("Amount in must be greater than zero".to_string());
    }

    PAIR_STATE.with(|state| {
        let mut s = state.borrow_mut();

        let (reserve_in, reserve_out) = if token_in == s.token_a {
            (s.reserve_a, s.reserve_b)
        } else if token_in == s.token_b {
            (s.reserve_b, s.reserve_a)
        } else {
            return Err("Invalid token".to_string());
        };

        // Uniswap V2 constant product formula: x * y = k
        let amount_out = (amount_in * reserve_out) / (reserve_in + amount_in);

        if amount_out < min_amount_out {
            return Err("Slippage exceeded".to_string());
        }

        if token_in == s.token_a {
            s.reserve_a += amount_in;
            s.reserve_b -= amount_out;
        } else {
            s.reserve_b += amount_in;
            s.reserve_a -= amount_out;
        }

        Ok(amount_out)
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
