use candid::{CandidType, Deserialize, Principal};
use ic_cdk::api::call::call;
use ic_cdk::{query, update};
use std::collections::HashMap;

#[derive(CandidType, Deserialize, Clone)]
pub struct FactoryConfig {
    pub governance_principal: Principal,
    pub paused: bool,
    pub whitelist: Vec<Principal>,
    pub pair_canister: Principal,
}

thread_local! {
    static FACTORY_CONFIG: std::cell::RefCell<FactoryConfig> = std::cell::RefCell::new(
        FactoryConfig {
            governance_principal: Principal::anonymous(),
            paused: false,
            whitelist: vec![],
            pair_canister: Principal::anonymous(),
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
async fn create_pair(token_a: Principal, token_b: Principal) -> Result<Principal, String> {
    let cfg = FACTORY_CONFIG.with(|config| config.borrow().clone());

    if cfg.paused {
        return Err("Factory is paused".to_string());
    }

    let caller = ic_cdk::api::caller();
    if caller != cfg.governance_principal && !cfg.whitelist.contains(&caller) {
        return Err("Unauthorized".to_string());
    }

    let (t_a, t_b) = if token_a < token_b {
        (token_a, token_b)
    } else {
        (token_b, token_a)
    };

    if let Some(existing) = PAIRS.with(|pairs| pairs.borrow().get(&(t_a, t_b)).cloned()) {
        return Ok(existing);
    }

    let (configure_result,): (Result<(), String>,) =
        call(cfg.pair_canister, "configure", (t_a, t_b))
            .await
            .map_err(|(code, message)| {
                format!("Failed to configure pair canister ({code:?}): {message}")
            })?;
    configure_result?;

    let pair_id = cfg.pair_canister;
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

fn main() {}
