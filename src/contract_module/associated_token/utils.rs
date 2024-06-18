use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

pub fn get_associated_token_address(mint: &Pubkey, owner: &Pubkey) -> Pubkey {
    // let usdc_dev_mint = Pubkey::from_str("Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr").unwrap();
    let associated_token_program = Pubkey::from_str("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL").unwrap();
    let (addr, _) = Pubkey::find_program_address(
        &[owner.as_ref(), spl_token::id().as_ref(), mint.as_ref()],
        &associated_token_program
    );
    addr
}