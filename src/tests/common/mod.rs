#![allow(dead_code)]
use crate::*;
pub struct Common {
    pub payer: Keypair,
    pub vl: VentureLaunch,
}

impl Common {
    pub fn new() -> Common {
        Common {
            payer: Keypair::read_from_file("./keypair.json").unwrap(), 
            vl: VentureLaunch::new(
                RpcClient::new("http://localhost:8899"),
                Pubkey::from_str("B1Lmegd5rBAAZ4nBRN9ePeMcThLdEQ5ec3yfDZZJxnBY").unwrap(),
                Pubkey::from_str("9Rv7CwWpLpF2F6iQirRQWcjcVYFGvxE6prPc2XZ3fFx1").unwrap(),
                Pubkey::from_str("E9ngGXiDDVLFR68vkQb8wJBGbPEfAxV68Gg2e5NjTB5P").unwrap(),
                spl_token::native_mint::id(),
            ),
        }
    }
}