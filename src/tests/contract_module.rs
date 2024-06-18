#![cfg(test)]

mod common;
use crate::*;
use common::Common;

// cargo test -- --test-threads=1

#[test]
#[ignore]
fn create_vault() {
    let mut common = Common::new();

    println!("Invoking create_vault...");
    let signature = common.vl.invoke_create_vault(
        &common.payer,
    ).unwrap();

    println!("[create_vault] Signature: {:?}", signature);
}

#[test]
fn deposit() {
    let common = Common::new();

    let balance = common.vl.get_vault_balance().unwrap();
    let amount = 3 * 10_u64.pow(9);

    println!("Invoking deposit...");
    let signature = common.vl.invoke_deposit(
        &common.payer,
        &native_mint_ata::get_associated_token_address(&common.vl.mint, &common.payer.pubkey()),
        amount
    ).unwrap();

    println!("[deposit] Signature: {:?}", signature);
    
    let new_balance = common.vl.get_vault_balance().unwrap();

    assert_eq!(new_balance - balance, amount);
}

#[test]
fn withdraw() {
    let common = Common::new();

    let balance = common.vl.get_vault_balance().unwrap();
    let amount = 3 * 10_u64.pow(9);

    println!("Invoking withdraw...");
    let signature = common.vl.invoke_withdraw(
        &common.payer,
        &native_mint_ata::get_associated_token_address(&common.vl.mint, &common.payer.pubkey()),
        amount
    ).unwrap();

    let new_balance = common.vl.get_vault_balance().unwrap();

    println!("[withdraw] Signature: {:?}", signature);

    assert_eq!(balance - new_balance, amount);
}