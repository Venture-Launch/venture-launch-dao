use std::future::Future;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    transaction::Transaction
};
use squads_multisig::{
    squads_multisig_program::Multisig,
    state::{
        Member,
        ProposalStatus
    }
};
use async_trait::async_trait;

#[async_trait]
pub trait BaseMultisig<Args> {
    type Error;

    async fn new(args: Args) -> Result<Self, Self::Error>
    where Self: Sized;

    fn get_multisig_create_args(&self) -> Args;
    async fn get_multisig(&self)                      -> Result<Multisig,        Self::Error>;
    async fn get_multisig_members(&self)              -> Result<Vec<Member>,     Self::Error>{
        let multisig = self.get_multisig().await?;
        Ok(multisig.members)
    }
    async fn get_multisig_transaction_index(&self)    -> Result<u64,             Self::Error>{
        let multisig = self.get_multisig().await?;
        Ok(multisig.transaction_index)
    }
    async fn get_threshold(&self)                     -> Result<u16,             Self::Error>{
        let multisig = self.get_multisig().await?;
        Ok(multisig.threshold)
    }
    async fn is_member(&self, member_pubkey: Pubkey)  -> Result<bool,            Self::Error>{
        let multisig = self.get_multisig().await?;
        Ok(multisig.is_member(member_pubkey).is_some())
    }
    async fn get_current_proposal_status(&self)       -> Result<ProposalStatus,  Self::Error>;

    async fn get_transaction_from_instructions(&self, sender: Pubkey, instructions: &[Instruction]) -> Result<Transaction, Self::Error>;
}