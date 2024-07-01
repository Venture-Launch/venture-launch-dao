#![cfg(test)]

use solana_sdk::{
    program_pack::Pack, pubkey::Pubkey, signature::{Keypair, Signer}, signer::EncodableKey
};
use solana_client::rpc_client::RpcClient;
use std::str::FromStr;
use dotenv::dotenv;

use crate::contract_module::{
    venture_launch::VentureLaunch,
    associated_token,
};

use crypto_tracker::state::CryptoTracker;

#[test]
fn contract_module_test() {
    dotenv().ok();
    let native_mint = spl_token::native_mint::id();

    let mut vl = VentureLaunch::new(
        RpcClient::new("http://localhost:8899"),
        Pubkey::from_str("B1Lmegd5rBAAZ4nBRN9ePeMcThLdEQ5ec3yfDZZJxnBY").unwrap(),
        Pubkey::from_str("8wd6uprrkJfgZdfu8PaAX6N1ZVJJStAtebdzgpDevJij").unwrap(),
        Pubkey::from_str("ApkXb5ayRzpxnH8yaVLgzEbLD4jhoGVzEWw5HTML9tcv").unwrap(),
        native_mint
    );

    // Initialize a keypair for the payer
    let payer = Keypair::read_from_file(std::env::var("KEYPAIR_PATH").expect("KEYPAIR_PATH must be set.")).unwrap();
    println!("Payer: {:?}", payer.pubkey());


    // Create & deposit to native ATA
    // println!("Creating ATA...");
    // associated_token::native::create_native_sol_ata(&vl.rpc_client, &payer);
    // println!("ATA created, depositing...");
    // associated_token::native::deposit_to_wrapped_sol_ata(&vl.rpc_client, &payer, 100 * 10_u64.pow(9));
    // println!("ATA deposited");

    // Contract initialization transaction
    println!("Invoking create_vault...");
    let signature = vl.invoke_create_vault(&payer).unwrap();
    println!("vault_account: {}", vl.vault_account);
    println!("data_account: {}", vl.data_account);

    println!("[create_vault] Signature: {:?}", signature);

    // Deposit to sthe vault
    println!("Invoking deposit...");
    let signature = vl.invoke_deposit(
        &payer,
        &associated_token::utils::get_associated_token_address(&native_mint, &payer.pubkey()),
        2 * 10_u64.pow(9)
    ).unwrap();

    println!("[deposit] Signature: {:?}", signature);

    // Withdraw from the vault
    println!("Invoking withdraw...");
    let signature = vl.invoke_withdraw(
        &payer,
        &associated_token::utils::get_associated_token_address(&native_mint, &payer.pubkey()),
        4 * 10_u64.pow(9)
    ).unwrap();

    println!("[withdraw] Signature: {:?}", signature);

    // Check account data
    let raw_data = vl.rpc_client.get_account_data(&vl.data_account).unwrap();
    let data = CryptoTracker::unpack_unchecked(&raw_data).unwrap();
    println!("Vault data:");
    println!("is_initialized: {}", data.is_initialized);
    println!("initializer_pubkey: {}", data.initializer_pubkey);
    println!("vault_account_pubkey: {}", data.vault_account_pubkey);
    println!("amount: {}", data.amount);
}