use candid::{CandidType, Deserialize, Nat, Principal};
use ic_cdk::{query, update};
use std::cell::RefCell;
use std::collections::BTreeMap;

#[derive(CandidType, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Account {
    pub owner: Principal,
    pub subaccount: Option<Vec<u8>>,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct InitialBalance {
    pub account: Account,
    pub amount: Nat,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct TokenInitArgs {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub initial_balances: Vec<InitialBalance>,
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

#[derive(CandidType, Deserialize, Clone)]
pub struct ApproveArgs {
    pub from_subaccount: Option<Vec<u8>>,
    pub spender: Account,
    pub amount: Nat,
    pub expected_allowance: Option<Nat>,
    pub expires_at: Option<u64>,
    pub fee: Option<Nat>,
    pub memo: Option<Vec<u8>>,
    pub created_at_time: Option<u64>,
}

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

#[derive(CandidType, Deserialize, Debug)]
pub enum ApproveError {
    BadFee { expected_fee: Nat },
    InsufficientFunds { balance: Nat },
    AllowanceChanged { current_allowance: Nat },
    TooOld,
    CreatedInFuture { ledger_time: u64 },
    TemporarilyUnavailable,
    Duplicate { duplicate_of: Nat },
    GenericError { error_code: Nat, message: String },
}

#[derive(CandidType, Deserialize, Debug)]
pub enum TransferFromError {
    BadFee { expected_fee: Nat },
    BadBurn { min_burn_amount: Nat },
    InsufficientFunds { balance: Nat },
    InsufficientAllowance { allowance: Nat },
    TooOld,
    CreatedInFuture { ledger_time: u64 },
    TemporarilyUnavailable,
    Duplicate { duplicate_of: Nat },
    GenericError { error_code: Nat, message: String },
}

#[derive(CandidType, Deserialize, Clone)]
pub struct AllowanceQuery {
    pub account: Account,
    pub spender: Account,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct AllowanceResult {
    pub allowance: Nat,
    pub expires_at: Option<u64>,
}

type TransferResult = Result<Nat, TransferError>;
type ApproveResult = Result<Nat, ApproveError>;
type TransferFromResult = Result<Nat, TransferFromError>;

#[derive(Clone, Default)]
struct TokenMetadata {
    name: String,
    symbol: String,
    decimals: u8,
    total_supply: u128,
}

thread_local! {
    static METADATA: RefCell<TokenMetadata> = RefCell::new(TokenMetadata::default());
    static BALANCES: RefCell<BTreeMap<Account, u128>> = const { RefCell::new(BTreeMap::new()) };
    static ALLOWANCES: RefCell<BTreeMap<(Account, Account), u128>> = const { RefCell::new(BTreeMap::new()) };
    static NEXT_TX_INDEX: RefCell<u128> = const { RefCell::new(0) };
}

fn nat_to_u128(value: &Nat) -> Result<u128, String> {
    // candid::Nat Display inserts `_` digit separators; strip before parsing.
    value
        .to_string()
        .replace('_', "")
        .parse::<u128>()
        .map_err(|_| "amount exceeds u128 range".to_string())
}

fn caller_account(subaccount: Option<Vec<u8>>) -> Account {
    Account {
        owner: ic_cdk::api::caller(),
        subaccount,
    }
}

fn next_tx_index() -> Nat {
    NEXT_TX_INDEX.with(|idx| {
        let mut tx = idx.borrow_mut();
        let current = *tx;
        *tx = current.saturating_add(1);
        Nat::from(current)
    })
}

fn get_balance(account: &Account) -> u128 {
    BALANCES.with(|balances| balances.borrow().get(account).copied().unwrap_or(0))
}

fn set_balance(account: Account, amount: u128) {
    BALANCES.with(|balances| {
        if amount == 0 {
            balances.borrow_mut().remove(&account);
        } else {
            balances.borrow_mut().insert(account, amount);
        }
    });
}

fn transfer_balance(from: &Account, to: &Account, amount: u128) -> Result<(), TransferError> {
    let from_balance = get_balance(from);
    if from_balance < amount {
        return Err(TransferError::InsufficientFunds {
            balance: Nat::from(from_balance),
        });
    }

    let to_balance = get_balance(to);
    let new_to = to_balance
        .checked_add(amount)
        .ok_or_else(|| TransferError::GenericError {
            error_code: Nat::from(1u8),
            message: "balance overflow".to_string(),
        })?;

    set_balance(from.clone(), from_balance - amount);
    set_balance(to.clone(), new_to);
    Ok(())
}

#[ic_cdk::init]
fn init(args: TokenInitArgs) {
    METADATA.with(|metadata| {
        *metadata.borrow_mut() = TokenMetadata {
            name: args.name,
            symbol: args.symbol,
            decimals: args.decimals,
            total_supply: 0,
        };
    });

    BALANCES.with(|balances| balances.borrow_mut().clear());
    ALLOWANCES.with(|allowances| allowances.borrow_mut().clear());
    NEXT_TX_INDEX.with(|idx| *idx.borrow_mut() = 0);

    for initial in args.initial_balances {
        let amount = match nat_to_u128(&initial.amount) {
            Ok(value) => value,
            Err(message) => ic_cdk::trap(&format!("invalid initial balance amount: {message}")),
        };
        if amount == 0 {
            continue;
        }

        let current = get_balance(&initial.account);
        let new_balance = current
            .checked_add(amount)
            .unwrap_or_else(|| ic_cdk::trap("initial balance overflow"));
        set_balance(initial.account, new_balance);

        METADATA.with(|metadata| {
            let mut data = metadata.borrow_mut();
            data.total_supply = data
                .total_supply
                .checked_add(amount)
                .unwrap_or_else(|| ic_cdk::trap("total supply overflow"));
        });
    }
}

#[query]
fn icrc1_name() -> String {
    METADATA.with(|metadata| metadata.borrow().name.clone())
}

#[query]
fn icrc1_symbol() -> String {
    METADATA.with(|metadata| metadata.borrow().symbol.clone())
}

#[query]
fn icrc1_decimals() -> u8 {
    METADATA.with(|metadata| metadata.borrow().decimals)
}

#[query]
fn icrc1_total_supply() -> Nat {
    METADATA.with(|metadata| Nat::from(metadata.borrow().total_supply))
}

#[query]
fn icrc1_balance_of(account: Account) -> Nat {
    Nat::from(get_balance(&account))
}

#[query]
fn icrc2_allowance(query: AllowanceQuery) -> AllowanceResult {
    let allowance = ALLOWANCES.with(|allowances| {
        allowances
            .borrow()
            .get(&(query.account, query.spender))
            .copied()
            .unwrap_or(0)
    });

    AllowanceResult {
        allowance: Nat::from(allowance),
        expires_at: None,
    }
}

#[update]
fn icrc1_transfer(args: TransferArg) -> TransferResult {
    let amount = match nat_to_u128(&args.amount) {
        Ok(value) => value,
        Err(message) => {
            return Err(TransferError::GenericError {
                error_code: Nat::from(2u8),
                message,
            });
        }
    };

    if amount == 0 {
        return Ok(next_tx_index());
    }

    let from = caller_account(args.from_subaccount);
    transfer_balance(&from, &args.to, amount)?;
    Ok(next_tx_index())
}

#[update]
fn icrc2_approve(args: ApproveArgs) -> ApproveResult {
    let amount = match nat_to_u128(&args.amount) {
        Ok(value) => value,
        Err(message) => {
            return Err(ApproveError::GenericError {
                error_code: Nat::from(3u8),
                message,
            });
        }
    };

    let owner = caller_account(args.from_subaccount);
    if get_balance(&owner) == 0 {
        return Err(ApproveError::InsufficientFunds {
            balance: Nat::from(0u8),
        });
    }

    if let Some(expected_allowance) = args.expected_allowance {
        let expected =
            nat_to_u128(&expected_allowance).map_err(|message| ApproveError::GenericError {
                error_code: Nat::from(4u8),
                message,
            })?;

        let current = ALLOWANCES.with(|allowances| {
            allowances
                .borrow()
                .get(&(owner.clone(), args.spender.clone()))
                .copied()
                .unwrap_or(0)
        });
        if current != expected {
            return Err(ApproveError::AllowanceChanged {
                current_allowance: Nat::from(current),
            });
        }
    }

    ALLOWANCES.with(|allowances| {
        allowances
            .borrow_mut()
            .insert((owner, args.spender), amount);
    });
    Ok(next_tx_index())
}

#[update]
fn icrc2_transfer_from(args: TransferFromArgs) -> TransferFromResult {
    let amount = match nat_to_u128(&args.amount) {
        Ok(value) => value,
        Err(message) => {
            return Err(TransferFromError::GenericError {
                error_code: Nat::from(5u8),
                message,
            });
        }
    };

    if amount == 0 {
        return Ok(next_tx_index());
    }

    let spender = caller_account(args.spender_subaccount);
    if spender != args.from {
        let key = (args.from.clone(), spender.clone());
        let allowance = ALLOWANCES.with(|allowances| allowances.borrow().get(&key).copied());
        let allowance = allowance.unwrap_or(0);

        if allowance < amount {
            return Err(TransferFromError::InsufficientAllowance {
                allowance: Nat::from(allowance),
            });
        }

        ALLOWANCES.with(|allowances| {
            allowances.borrow_mut().insert(key, allowance - amount);
        });
    }

    let from_balance = get_balance(&args.from);
    if from_balance < amount {
        return Err(TransferFromError::InsufficientFunds {
            balance: Nat::from(from_balance),
        });
    }

    let to_balance = get_balance(&args.to);
    let new_to = to_balance
        .checked_add(amount)
        .ok_or_else(|| TransferFromError::GenericError {
            error_code: Nat::from(6u8),
            message: "balance overflow".to_string(),
        })?;

    set_balance(args.from, from_balance - amount);
    set_balance(args.to, new_to);
    Ok(next_tx_index())
}

ic_cdk::export_candid!();

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn nat_to_u128_handles_underscore_separators() {
        let value = Nat::from(1_000_000_000_000_000u128);
        assert_eq!(nat_to_u128(&value).unwrap(), 1_000_000_000_000_000u128);
    }

    #[test]
    fn candid_interface_matches_checked_in_did() {
        let generated_candid = __export_service();
        let did_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("mock_token.did");
        let checked_in_candid =
            std::fs::read_to_string(&did_path).expect("failed to read checked-in mock_token.did");

        assert_eq!(
            generated_candid.trim_end(),
            checked_in_candid.trim_end(),
            "mock_token.did is out of date"
        );
    }
}
