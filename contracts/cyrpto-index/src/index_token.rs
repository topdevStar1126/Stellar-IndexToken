use soroban_sdk::{contract, contractimpl, Address, Env, String};
use soroban_token_sdk::{metadata::TokenMetadata, TokenUtils};
use crate::allowance::{read_allowance, spend_allowance, write_allowance};
use crate::balance::{read_balance, receive_balance, spend_balance};
use crate::storage_types::{DataKey, INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD};
use crate::metadata::{read_name, read_decimal, read_symbol, write_metadata};
use crate::admin::{has_administrator, write_administrator, read_administrator};
// Define the IndexToken contract
#[contract]
pub struct IndexToken;

// Implement the contract's methods
#[contractimpl]
impl IndexToken {
    pub fn initialize(e: Env, admin: Address, decimal: u32, name: String, symbol: String) {
        if has_administrator(&e) {
            panic!("already initialized")
        }
        write_administrator(&e, &admin);
        if decimal > 18 {
            panic!("Decimal must not be greater than 18");
        }

        write_metadata(
            &e,
            TokenMetadata {
                decimal,
                name,
                symbol,
            },
        )
    }

    pub fn add_token(e: Env, token_address: Address, amount: i128) {
        let admin = read_administrator(&e);
        admin.require_auth();
        let token_address_clone1 = token_address.clone();
        let token_address_clone2 = token_address.clone();
        // Here you might implement logic to interact with the specific token contract
        // For now, this is a placeholder
        let token_balance = e.storage().instance().get::<_, i128>(&DataKey::Balance(token_address)).unwrap_or(0);
        e.storage().instance().set(&DataKey::Balance(token_address_clone1), &(token_balance + amount));

        TokenUtils::new(&e).events().mint(admin, token_address_clone2, amount);
    }

    pub fn remove_token(e: Env, token_address: Address, amount: i128) {
        let admin = read_administrator(&e);
        admin.require_auth();
        let token_address_clone1 = token_address.clone();
        // Here you might implement logic to interact with the specific token contract
        // For now, this is a placeholder
        let token_balance = e.storage().instance().get::<_, i128>(&DataKey::Balance(token_address)).unwrap_or(0);
        if token_balance < amount {
            panic!("insufficient token balance");
        }

        e.storage().instance().set(&DataKey::Balance(token_address_clone1), &(token_balance - amount));

        TokenUtils::new(&e).events().burn(admin, amount);
    }

    pub fn get_token_balance(e: Env, token_address: Address) -> i128 {
        e.storage().instance().get::<_, i128>(&DataKey::Balance(token_address)).unwrap_or(0)
    }

    pub fn update_nav(e: Env, new_nav: i128) {
        let admin = read_administrator(&e);
        admin.require_auth();
        e.storage().instance().set(&DataKey::NAV, &new_nav);
    }

    pub fn get_nav(e: Env) -> i128 {
        e.storage().instance().get::<_, i128>(&DataKey::NAV).unwrap_or(0)
    }
}

impl soroban_sdk::token::Interface for IndexToken {
    fn allowance(e: Env, from: Address, spender: Address) -> i128 {
        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        read_allowance(&e, from, spender).amount
    }

    fn approve(e: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32) {
        from.require_auth();

        check_nonnegative_amount(amount);

        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        write_allowance(&e, from.clone(), spender.clone(), amount, expiration_ledger);
        TokenUtils::new(&e)
            .events()
            .approve(from, spender, amount, expiration_ledger);
    }

    fn balance(e: Env, id: Address) -> i128 {
        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        read_balance(&e, id)
    }

    fn transfer(e: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();

        check_nonnegative_amount(amount);

        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        spend_balance(&e, from.clone(), amount);
        receive_balance(&e, to.clone(), amount);
        TokenUtils::new(&e).events().transfer(from, to, amount);
    }

    fn transfer_from(e: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();

        check_nonnegative_amount(amount);

        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        spend_allowance(&e, from.clone(), spender, amount);
        spend_balance(&e, from.clone(), amount);
        receive_balance(&e, to.clone(), amount);
        TokenUtils::new(&e).events().transfer(from, to, amount)
    }

    fn burn(e: Env, from: Address, amount: i128) {
        from.require_auth();

        check_nonnegative_amount(amount);

        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        spend_balance(&e, from.clone(), amount);
        TokenUtils::new(&e).events().burn(from, amount);
    }

    fn burn_from(e: Env, spender: Address, from: Address, amount: i128) {
        spender.require_auth();

        check_nonnegative_amount(amount);

        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        spend_allowance(&e, from.clone(), spender, amount);
        spend_balance(&e, from.clone(), amount);
        TokenUtils::new(&e).events().burn(from, amount)
    }

    fn decimals(e: Env) -> u32 {
        read_decimal(&e)
    }

    fn name(e: Env) -> String {
        read_name(&e)
    }

    fn symbol(e: Env) -> String {
        read_symbol(&e)
    }
}

// Helper function to check non-negative amounts
fn check_nonnegative_amount(amount: i128) {
    if amount < 0 {
        panic!("negative amount is not allowed: {}", amount)
    }
}
