use candid::{CandidType, Deserialize, Principal};
use ic_cdk::{query, update};
use std::collections::{BTreeMap, BTreeSet};

#[derive(CandidType, Deserialize, Clone)]
pub struct FactoryConfig {
    pub governance_principal: Principal,
    pub paused: bool,
    pub whitelist: Vec<Principal>,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct PairRecord {
    pub token_a: Principal,
    pub token_b: Principal,
    pub pair_canister: Principal,
}

thread_local! {
    static FACTORY_CONFIG: std::cell::RefCell<FactoryConfig> = std::cell::RefCell::new(
        FactoryConfig {
            governance_principal: Principal::anonymous(),
            paused: false,
            whitelist: vec![],
        }
    );
    static PAIRS: std::cell::RefCell<BTreeMap<(Principal, Principal), Principal>> = std::cell::RefCell::new(BTreeMap::new());
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

fn require_governance() -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    FACTORY_CONFIG.with(|config| {
        let cfg = config.borrow();
        if caller != cfg.governance_principal {
            return Err("Unauthorized".to_string());
        }
        Ok(())
    })
}

fn ensure_pair_creation_allowed(token_a: Principal, token_b: Principal) -> Result<(), String> {
    FACTORY_CONFIG.with(|config| {
        let cfg = config.borrow();
        if cfg.paused {
            return Err("Factory is paused".to_string());
        }

        if cfg.whitelist.is_empty() {
            return Ok(());
        }

        let allowed: BTreeSet<Principal> = cfg.whitelist.iter().cloned().collect();
        if !allowed.contains(&token_a) || !allowed.contains(&token_b) {
            return Err("One or both tokens are not whitelisted".to_string());
        }

        Ok(())
    })
}

#[ic_cdk::init]
fn init(config: FactoryConfig) {
    FACTORY_CONFIG.with(|c| *c.borrow_mut() = config);
}

#[update]
fn create_pair(
    token_a: Principal,
    token_b: Principal,
    pair_canister: Principal,
) -> Result<Principal, String> {
    require_governance()?;

    if pair_canister == Principal::anonymous() {
        return Err("pair_canister cannot be anonymous".to_string());
    }

    let (t_a, t_b) = canonical_tokens(token_a, token_b)?;
    ensure_pair_creation_allowed(t_a, t_b)?;

    PAIRS.with(|pairs| {
        let mut pairs_mut = pairs.borrow_mut();
        if pairs_mut.contains_key(&(t_a, t_b)) {
            return Err("Pair already exists".to_string());
        }

        if pairs_mut
            .values()
            .any(|existing| *existing == pair_canister)
        {
            return Err("Pair canister is already registered".to_string());
        }

        pairs_mut.insert((t_a, t_b), pair_canister);
        Ok(pair_canister)
    })
}

#[update]
fn remove_pair(token_a: Principal, token_b: Principal) -> Result<(), String> {
    require_governance()?;
    let (t_a, t_b) = canonical_tokens(token_a, token_b)?;

    PAIRS.with(|pairs| {
        let removed = pairs.borrow_mut().remove(&(t_a, t_b));
        if removed.is_none() {
            return Err("Pair not found".to_string());
        }
        Ok(())
    })
}

#[query]
fn list_pairs() -> Vec<PairRecord> {
    PAIRS.with(|pairs| {
        pairs
            .borrow()
            .iter()
            .map(|((token_a, token_b), pair_canister)| PairRecord {
                token_a: *token_a,
                token_b: *token_b,
                pair_canister: *pair_canister,
            })
            .collect()
    })
}

#[query]
fn get_pair(token_a: Principal, token_b: Principal) -> Option<Principal> {
    let (t_a, t_b) = canonical_tokens(token_a, token_b).ok()?;
    PAIRS.with(|pairs| pairs.borrow().get(&(t_a, t_b)).cloned())
}

#[query]
fn get_config() -> FactoryConfig {
    FACTORY_CONFIG.with(|c| c.borrow().clone())
}

#[update]
fn set_paused(paused: bool) -> Result<(), String> {
    require_governance()?;
    FACTORY_CONFIG.with(|config| {
        config.borrow_mut().paused = paused;
        Ok(())
    })
}

#[update]
fn set_whitelist(whitelist: Vec<Principal>) -> Result<(), String> {
    require_governance()?;
    FACTORY_CONFIG.with(|config| {
        config.borrow_mut().whitelist = whitelist;
        Ok(())
    })
}

ic_cdk::export_candid!();

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn canonical_tokens_are_order_independent() {
        let token_a = Principal::from_text("2vxsx-fae").unwrap();
        let token_b = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();

        assert_eq!(
            canonical_tokens(token_a, token_b),
            canonical_tokens(token_b, token_a)
        );
    }

    #[test]
    fn candid_interface_matches_checked_in_did() {
        let generated_candid = __export_service();
        let did_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("amm_factory.did");
        let checked_in_candid =
            std::fs::read_to_string(&did_path).expect("failed to read checked-in amm_factory.did");

        assert_eq!(
            generated_candid.trim_end(),
            checked_in_candid.trim_end(),
            "amm_factory.did is out of date"
        );
    }
}
