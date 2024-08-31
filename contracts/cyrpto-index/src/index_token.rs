use crate::admin::{has_administrator, read_administrator, write_administrator};
use crate::allowance::{read_allowance, spend_allowance, write_allowance};
use crate::balance::{is_authorized, write_authorization};
use crate::balance::{read_balance, receive_balance, spend_balance};
use crate::event;
use crate::metadata::{read_decimal, read_name, read_symbol, write_metadata};
use soroban_sdk::{contractimpl, Address, Bytes, Env, String};
use soroban_token_sdk::TokenMetadata;

pub trait IndexTokenTrait {
    fn initialize_index(e: Env, admin: Address, decimal: u32, name: String, symbol: String);

    fn add_asset(e: Env, asset: Address, amount: i128);

    fn remove_asset(e: Env, asset: Address, amount: i128);

    fn get_asset_amount(e: Env, asset: Address) -> i128;

    fn set_admin(e: Env, new_admin: Address);
}

#[contract]
pub struct IndexToken;

#[contractimpl]
impl IndexTokenTrait for IndexToken {
    fn initialize_index(e: Env, admin: Address, decimal: u32, name: String, symbol: String) {
        if has_administrator(&e) {
            panic!("already initialized");
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
        );
    }

    fn add_asset(e: Env, asset: Address, amount: i128) {
        asset.require_auth();
        check_nonnegative_amount(amount);

        let admin = read_administrator(&e);
        admin.require_auth();

        let asset_balance = read_balance(&e, asset.clone());
        receive_balance(&e, asset, amount);
        event::mint(&e, admin, asset, amount);
    }

    fn remove_asset(e: Env, asset: Address, amount: i128) {
        asset.require_auth();
        check_nonnegative_amount(amount);

        let admin = read_administrator(&e);
        admin.require_auth();

        spend_balance(&e, asset.clone(), amount);
        event::burn(&e, asset, amount);
    }

    fn get_asset_amount(e: Env, asset: Address) -> i128 {
        read_balance(&e, asset)
    }

    fn set_admin(e: Env, new_admin: Address) {
        let admin = read_administrator(&e);
        admin.require_auth();
        write_administrator(&e, &new_admin);
        event::set_admin(&e, admin, new_admin);
    }
}

fn check_nonnegative_amount(amount: i128) {
    if amount < 0 {
        panic!("negative amount is not allowed: {}", amount);
    }
}
