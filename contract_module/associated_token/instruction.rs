use solana_sdk::{
    instruction::{AccountMeta, Instruction}, pubkey::Pubkey
};
use std::str::FromStr;

pub fn create_associated_token_program_instruction(payer: &Pubkey, ata: &Pubkey, mint: &Pubkey) -> Instruction {
    let associated_token_program = Pubkey::from_str("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL").unwrap();
    let instruction_data = vec![];
    let create_associated_token_program_instruction = solana_sdk::instruction::Instruction::new_with_bytes(
        associated_token_program.clone(),
        &instruction_data[..],
        vec![
            AccountMeta::new(payer.clone(), true),
            AccountMeta::new(ata.clone(), false),
            AccountMeta::new_readonly(payer.clone(), false),
            AccountMeta::new_readonly(mint.clone(), false),
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
    );
    create_associated_token_program_instruction
}