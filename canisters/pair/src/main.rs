use candid::{CandidType, Deserialize, Principal};
use ic_cdk::{query, update};
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(CandidType, Deserialize, Clone)]
pub struct PairState {
    pub token_a: Principal,
    pub token_b: Principal,
    pub reserve_a: u128,
    pub reserve_b: u128,
    pub total_supply: u128,
    pub balances: HashMap<Principal, u128>,
    pub configured: bool,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct PairSnapshot {
    pub token_a: Principal,
    pub token_b: Principal,
    pub reserve_a: u128,
    pub reserve_b: u128,
    pub total_supply: u128,
    pub configured: bool,
}

impl Default for PairState {
    fn default() -> Self {
        Self {
            token_a: Principal::anonymous(),
            token_b: Principal::anonymous(),
            reserve_a: 0,
            reserve_b: 0,
            total_supply: 0,
            balances: HashMap::new(),
            configured: false,
        }
    }
}

thread_local! {
    static PAIR_STATE: RefCell<PairState> = RefCell::new(PairState::default());
}

fn integer_sqrt(value: u128) -> u128 {
    if value == 0 {
        return 0;
    }

    let mut current = value;
    let mut next = (current + 1) / 2;

    while next < current {
        current = next;
        next = (current + value / current) / 2;
    }

    current
}

#[ic_cdk::init]
fn init() {}

#[update]
fn configure(token_a: Principal, token_b: Principal) -> Result<(), String> {
    configure_pair(token_a, token_b)
}

fn configure_pair(token_a: Principal, token_b: Principal) -> Result<(), String> {
    if token_a == token_b {
        return Err("Tokens must be different".to_string());
    }

    PAIR_STATE.with(|state| {
        let mut s = state.borrow_mut();

        if s.configured {
            if s.token_a == token_a && s.token_b == token_b {
                return Ok(());
            }
            return Err("Pair already configured".to_string());
        }

        s.token_a = token_a;
        s.token_b = token_b;
        s.configured = true;
        Ok(())
    })
}

#[update]
fn add_liquidity(amount_a: u128, amount_b: u128) -> Result<u128, String> {
    add_liquidity_for(ic_cdk::api::caller(), amount_a, amount_b)
}

fn add_liquidity_for(caller: Principal, amount_a: u128, amount_b: u128) -> Result<u128, String> {
    if amount_a == 0 || amount_b == 0 {
        return Err("Amounts must be greater than zero".to_string());
    }

    PAIR_STATE.with(|state| {
        let mut s = state.borrow_mut();
        if !s.configured {
            return Err("Pair not configured".to_string());
        }

        let lp_tokens = if s.total_supply == 0 {
            integer_sqrt(amount_a.saturating_mul(amount_b))
        } else {
            if s.reserve_a == 0 || s.reserve_b == 0 {
                return Err("Pair reserves are empty".to_string());
            }
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
    remove_liquidity_for(ic_cdk::api::caller(), lp_amount)
}

fn remove_liquidity_for(caller: Principal, lp_amount: u128) -> Result<(u128, u128), String> {
    if lp_amount == 0 {
        return Err("LP amount must be greater than zero".to_string());
    }

    PAIR_STATE.with(|state| {
        let mut s = state.borrow_mut();
        if !s.configured {
            return Err("Pair not configured".to_string());
        }

        let balance = s.balances.get(&caller).copied().unwrap_or(0);
        if balance < lp_amount {
            return Err("Insufficient LP tokens".to_string());
        }

        if s.total_supply == 0 {
            return Err("No liquidity available".to_string());
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
    swap_for(token_in, amount_in, min_amount_out)
}

fn swap_for(token_in: Principal, amount_in: u128, min_amount_out: u128) -> Result<u128, String> {
    if amount_in == 0 {
        return Err("Amount in must be greater than zero".to_string());
    }

    PAIR_STATE.with(|state| {
        let mut s = state.borrow_mut();
        if !s.configured {
            return Err("Pair not configured".to_string());
        }

        let (reserve_in, reserve_out) = if token_in == s.token_a {
            (s.reserve_a, s.reserve_b)
        } else if token_in == s.token_b {
            (s.reserve_b, s.reserve_a)
        } else {
            return Err("Invalid token".to_string());
        };

        if reserve_in == 0 || reserve_out == 0 {
            return Err("Insufficient liquidity".to_string());
        }

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
fn get_snapshot() -> PairSnapshot {
    PAIR_STATE.with(|state| {
        let s = state.borrow();
        PairSnapshot {
            token_a: s.token_a,
            token_b: s.token_b,
            reserve_a: s.reserve_a,
            reserve_b: s.reserve_b,
            total_supply: s.total_supply,
            configured: s.configured,
        }
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

fn main() {}

#[cfg(test)]
mod tests {
    use super::*;

    fn reset_state() {
        PAIR_STATE.with(|state| {
            *state.borrow_mut() = PairState::default();
        });
    }

    #[test]
    fn integer_sqrt_handles_basic_values() {
        assert_eq!(integer_sqrt(0), 0);
        assert_eq!(integer_sqrt(1), 1);
        assert_eq!(integer_sqrt(4), 2);
        assert_eq!(integer_sqrt(15), 3);
        assert_eq!(integer_sqrt(16), 4);
    }

    #[test]
    fn lifecycle_flow_updates_snapshot_and_balances() {
        reset_state();

        let token_a = Principal::from_slice(&[1]);
        let token_b = Principal::from_slice(&[2]);
        let caller = Principal::from_slice(&[3]);

        configure_pair(token_a, token_b).expect("configure");
        assert!(get_snapshot().configured);

        let minted = add_liquidity_for(caller, 1_000, 1_000).expect("mint lp");
        assert_eq!(minted, 1_000);
        assert_eq!(get_reserves(), (1_000, 1_000));
        assert_eq!(total_supply(), 1_000);
        assert_eq!(get_balance(caller), 1_000);

        let amount_out = swap_for(token_a, 100, 1).expect("swap");
        assert!(amount_out > 0);
        assert_eq!(get_reserves().0, 1_100);

        let removed = remove_liquidity_for(caller, 100).expect("remove");
        assert!(removed.0 > 0);
        assert!(removed.1 > 0);
        assert_eq!(get_balance(caller), 900);
    }
}
