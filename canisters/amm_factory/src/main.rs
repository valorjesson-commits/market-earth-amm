use candid::{CandidType, Deserialize, Principal};
use ic_cdk::storage;
use ic_cdk::{query, update};
use std::collections::HashMap;

#[derive(CandidType, Deserialize, Clone)]
pub struct FactoryConfig {
    pub governance_principal: Principal,
    pub paused: bool,
    pub whitelist: Vec<Principal>,
}

thread_local! {
    static FACTORY_CONFIG: std::cell::RefCell<FactoryConfig> = std::cell::RefCell::new(
        FactoryConfig {
            governance_principal: Principal::anonymous(),
            paused: false,
            whitelist: vec![],
        }
    );
    static PAIRS: std::cell::RefCell<HashMap<(Principal, Principal), Principal>> = std::cell::RefCell::new(HashMap::new());
    static PAIR_LIST: std::cell::RefCell<Vec<(Principal, Principal, Principal)>> = std::cell::RefCell::new(vec![]);
}

#[ic_cdk::init]
fn init(config: FactoryConfig) {
    FACTORY_CONFIG.with(|c| *c.borrow_mut() = config);
}

#[update]
fn create_pair(token_a: Principal, token_b: Principal) -> Result<Principal, String> {
    FACTORY_CONFIG.with(|config| {
        let cfg = config.borrow();
        if cfg.paused {
            return Err("Factory is paused".to_string());
        }
        Ok(())
    })?;

    let (t_a, t_b) = if token_a < token_b {
        (token_a, token_b)
    } else {
        (token_b, token_a)
    };

    PAIRS.with(|pairs| {
        let mut p = pairs.borrow_mut();
        if p.contains_key(&(t_a, t_b)) {
            return Err("Pair already exists".to_string());
        }
        Ok(())
    })?;

    // Generate deterministic pair canister ID (simplified)
    let pair_id = Principal::from_slice(&ic_cdk::api::crypto::sha256(
        format!("pair-{}-{}", t_a, t_b).as_bytes(),
    )[0..29]);

    PAIRS.with(|pairs| {
        pairs.borrow_mut().insert((t_a, t_b), pair_id);
    });

    PAIR_LIST.with(|list| {
        list.borrow_mut().push((t_a, t_b, pair_id));
    });

    Ok(pair_id)
}

#[query]
fn list_pairs() -> Vec<(Principal, Principal, Principal)> {
    PAIR_LIST.with(|list| list.borrow().clone())
}

#[query]
fn get_pair(token_a: Principal, token_b: Principal) -> Option<Principal> {
    let (t_a, t_b) = if token_a < token_b {
        (token_a, token_b)
    } else {
        (token_b, token_a)
    };

    PAIRS.with(|pairs| pairs.borrow().get(&(t_a, t_b)).cloned())
}

#[query]
fn get_config() -> FactoryConfig {
    FACTORY_CONFIG.with(|c| c.borrow().clone())
}

#[update]
fn set_paused(paused: bool) -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    FACTORY_CONFIG.with(|config| {
        let mut cfg = config.borrow_mut();
        if caller != cfg.governance_principal {
            return Err("Unauthorized".to_string());
        }
        cfg.paused = paused;
        Ok(())
    })
}

ic_cdk::export_candid!();
