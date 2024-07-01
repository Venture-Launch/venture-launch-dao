use solana_sdk::{
    message::Message, signature::{Keypair, Signer}, transaction::Transaction
};
use solana_client::rpc_client::RpcClient;
use super::utils::get_associated_token_address;
use super::instruction;

pub fn create_native_sol_ata(rpc_client: &RpcClient, payer: &Keypair) {
    let ata_ix = instruction::create_associated_token_program_instruction(
    &payer.pubkey(), 
    &get_associated_token_address(&spl_token::native_mint::id(), &payer.pubkey()),
    &spl_token::native_mint::id()
    );
    let message = Message::new(
        &[ata_ix],
        Some(&payer.pubkey())
    );
    let recent_blockhash = rpc_client.get_latest_blockhash().unwrap();
    let mut tx = Transaction::new(
        &[&payer],
        message,
        recent_blockhash,
    );
    tx.sign(&[payer], recent_blockhash);
    rpc_client.send_and_confirm_transaction(&tx).unwrap();
}

pub fn deposit_to_wrapped_sol_ata(rpc_client: &RpcClient, payer: &Keypair, amount: u64) {
    let wrapped_solana_ata = get_associated_token_address(&spl_token::native_mint::id(), &payer.pubkey());
    println!("ATA: {wrapped_solana_ata}");

    let transfer_instruction = solana_sdk::system_instruction::transfer(
        &payer.pubkey(), 
        &wrapped_solana_ata,
        amount,
    );
    let native_sync_instruction = spl_token::instruction::sync_native(
        &spl_token::id(),
        &wrapped_solana_ata
    ).unwrap();

    let message = Message::new(
        &[transfer_instruction, native_sync_instruction],
        Some(&payer.pubkey())
    );
    let recent_blockhash = rpc_client.get_latest_blockhash().unwrap();
    let mut tx = Transaction::new(
        &[&payer],
        message,
        recent_blockhash,
    );
    tx.sign(&[payer], recent_blockhash);
    
    rpc_client.send_and_confirm_transaction(&tx).unwrap();
}