use solana_sdk::{
    instruction::{AccountMeta, Instruction}, pubkey::Pubkey
};

use super::venture_launch::VentureLaunch;

pub fn create_vault(
    vl: &VentureLaunch,
    payer: &Pubkey,
) -> Instruction {
    let instruction_data = vec![0];
    let create_vault_instruction = Instruction::new_with_bytes(
        vl.program_id.clone(),
        &instruction_data[..],
        vec![
            AccountMeta::new_readonly(payer.clone(), true),
            AccountMeta::new(vl.vault_account.clone(), false),
            AccountMeta::new(vl.data_account.clone(), false),
            AccountMeta::new_readonly(solana_sdk::sysvar::rent::id(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
    );
    return create_vault_instruction;
}

pub fn deposit(
    vl: &VentureLaunch,
    payer: &Pubkey,
    ata_account: &Pubkey,
    amount: u64
) -> Instruction {
    let amount_data = amount.to_le_bytes();
    let instruction_data = 
    std::iter::once(1)
    .chain(amount_data.into_iter())
    .collect::<Vec<_>>();

    let deposit_instruction = Instruction::new_with_bytes(
        vl.program_id.clone(),
        &instruction_data,
        vec![
            AccountMeta::new_readonly(payer.clone(), true),
            AccountMeta::new(ata_account.clone(), false),
            AccountMeta::new(vl.vault_account.clone(), false),
            AccountMeta::new(vl.data_account.clone(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
    );
    return deposit_instruction;
}

pub fn withdraw(
    vl: &VentureLaunch,
    payer: &Pubkey,
    receive_account: &Pubkey,
    amount: u64,
) -> Instruction {
    let amount_data = amount.to_le_bytes();
    let instruction_data = 
    std::iter::once(2)
    .chain(amount_data.into_iter())
    .collect::<Vec<_>>();

    let (pda_account, _) = Pubkey::find_program_address(&[b"cryptotracker"], &vl.program_id);
    
    let withdraw_instruction = Instruction::new_with_bytes(
        vl.program_id.clone(),
        &instruction_data,
        vec![
            AccountMeta::new_readonly(payer.clone(), true),
            AccountMeta::new(receive_account.clone(), false),
            AccountMeta::new(vl.vault_account.clone(), false),
            AccountMeta::new(vl.data_account.clone(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new(pda_account, false),
        ],
    );

    return withdraw_instruction;
}