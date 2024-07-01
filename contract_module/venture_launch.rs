use solana_sdk::{
    message::Message, pubkey::Pubkey, signature::{Keypair, Signer, Signature}, transaction::Transaction,
    program_pack::Pack
};
use solana_client::{
    rpc_client::RpcClient,
    client_error::Result as ClientResult,
};
use crypto_tracker::state::CryptoTracker;
use super::instruction;

const ACCOUNT_SIZE: u64 = 165;
const CRYPTO_TRACKER_DATA_SIZE: u64 = 73;

pub struct VentureLaunch {
    pub rpc_client: RpcClient,
    pub program_id: Pubkey,
    pub vault_account: Pubkey,
    pub data_account: Pubkey,
    pub mint: Pubkey,
}

impl VentureLaunch {
    pub fn new(
        rpc_client: RpcClient,
        program_id: Pubkey,
        vault_account: Pubkey,
        data_account: Pubkey,
        mint: Pubkey,
    ) -> VentureLaunch {
        VentureLaunch {
            rpc_client,
            program_id,
            vault_account,
            data_account,
            mint,
        }
    }

    pub fn invoke_create_vault(
        &mut self,
        payer: &Keypair,
    ) -> ClientResult<Signature> {
        let mut instructions = Vec::new();
    
        // Create token account that will be transfered to program
        let vault_account = Keypair::new();
        self.vault_account = vault_account.pubkey();
        instructions.push(solana_sdk::system_instruction::create_account(
            &payer.pubkey(),
            &self.vault_account,
            self.rpc_client.get_minimum_balance_for_rent_exemption(ACCOUNT_SIZE as usize).unwrap(),
            ACCOUNT_SIZE,
            &spl_token::id()
        ));
        instructions.push(spl_token::instruction::initialize_account(
            &spl_token::id(),
            &self.vault_account,
            &self.mint,
            &payer.pubkey()
        ).unwrap());
    
        // Create data account (to store state) that will be transfered to program
        let data_account = Keypair::new();
        self.data_account = data_account.pubkey();
        instructions.push(solana_sdk::system_instruction::create_account(
            &payer.pubkey(),
            &self.data_account,
            self.rpc_client.get_minimum_balance_for_rent_exemption(CRYPTO_TRACKER_DATA_SIZE as usize).unwrap(),
            CRYPTO_TRACKER_DATA_SIZE,
            &self.program_id
        ));
    
        // Add call to the smart contract
        instructions.push(instruction::create_vault(
            &self,
            &payer.pubkey()
        ));
    
        // Create tx
        let msg = Message::new(&instructions, Some(&payer.pubkey()));
        let mut tx = Transaction::new_unsigned(msg);
        let recent_blockhash = self.rpc_client.get_latest_blockhash().unwrap();
        tx.sign(
            &[&payer, &vault_account, &data_account],
            recent_blockhash
        );
    
        self.rpc_client.send_and_confirm_transaction(&tx)
    }
    
    pub fn invoke_deposit(
        &self,
        payer: &Keypair,
        deposit_account: &Pubkey,
        amount: u64
    ) -> ClientResult<Signature> {
        let deposit_instruction = instruction::deposit(
            &self,
            &payer.pubkey(),
            &deposit_account,
            amount
        );
        let tx = Transaction::new_signed_with_payer(
            &[deposit_instruction],
            Some(&payer.pubkey()),
            &[&payer],
            self.rpc_client.get_latest_blockhash().unwrap(),
        );

        self.rpc_client.send_and_confirm_transaction(&tx)
    }
    
    pub fn invoke_withdraw(
        &self,
        payer: &Keypair,
        withdraw_account: &Pubkey,
        amount: u64,
    ) -> ClientResult<Signature> {
        let withdraw_instruction = instruction::withdraw(
            &self,
            &payer.pubkey(),
            &withdraw_account,
            amount
        );
        let tx = Transaction::new_signed_with_payer(
            &[withdraw_instruction],
            Some(&payer.pubkey()),
            &[&payer],
            self.rpc_client.get_latest_blockhash().unwrap(),
        );
        self.rpc_client.send_and_confirm_transaction(&tx)
    }

    pub fn get_vault_balance(&self) -> ClientResult<u64> {
        let raw_data = self.rpc_client.get_account_data(&self.data_account)?;
        let data = CryptoTracker::unpack_unchecked(&raw_data).unwrap();
        println!("Balance: {}", data.amount);
        Ok(data.amount)
    }
}