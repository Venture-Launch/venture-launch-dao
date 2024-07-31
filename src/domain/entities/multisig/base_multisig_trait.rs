use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    instruction::Instruction, message::Message, pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction
};
use squads_multisig::{
    anchor_lang::AccountDeserialize, client::{proposal_approve, proposal_cancel, ProposalVoteAccounts, ProposalVoteArgs}, pda::{get_multisig_pda, get_program_config_pda, get_proposal_pda, get_vault_pda}, squads_multisig_program::{self, state::ProgramConfig, Multisig}, state::{
        Member, Proposal, ProposalStatus
    }
};
use async_trait::async_trait;

use super::{base_multisig::{BaseMultisig, BaseMultisigCreateArgs, BaseMultisigInitArgs}, error::BaseMultisigError};

#[async_trait]
pub trait BaseMultisigTrait<Args>: Send + Sync {
    type Error;

    async fn new(args: Args) -> Result<Self, Self::Error>
    where Self: Sized;
    async fn from_multisig_pda(args: BaseMultisigInitArgs) -> Result<Self, Self::Error>
    where Self: Sized;

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

    fn get_rpc_client(&self) -> &RpcClient;
    fn get_creator_key(&self) -> Pubkey;
    fn get_multisig_pda(&self) -> Pubkey;
    fn get_vault_pda(&self) -> Pubkey;
    fn get_program_config_pda(&self) -> Pubkey;
    fn get_treasury(&self) -> Pubkey;
    fn get_create_keypair(&self) -> &Option<Keypair>;

    async fn instruction_proposal_approve(&self, approver: Pubkey)  -> Result<Instruction, Self::Error>;
    async fn instruction_proposal_cancel(&self, canceler: Pubkey) -> Result<Instruction, Self::Error>;
    async fn transaction_proposal_approve(&self, approver: Pubkey)  -> Result<Transaction, Self::Error> {
        let ix = self.instruction_proposal_approve(approver).await?;

        Ok(self.get_transaction_from_instructions(approver, &[ix]).await?)

    }

    async fn transaction_proposal_cancel(&self, canceler: Pubkey) -> Result<Transaction, Self::Error> {
        let ix = self.instruction_proposal_cancel(canceler).await?;

        Ok(self.get_transaction_from_instructions(canceler, &[ix]).await?)
    }
}

#[async_trait]
impl BaseMultisigTrait<BaseMultisigCreateArgs> for BaseMultisig {
    type Error = BaseMultisigError;

    fn get_rpc_client(&self) -> &RpcClient {
        return &self.rpc_client;
    }
    fn get_creator_key(&self) -> Pubkey {
        return self.creator;
    }
    fn get_multisig_pda(&self) -> Pubkey {
        return self.multisig_pda;
    }
    fn get_vault_pda(&self) -> Pubkey {
        return self.vault_pda;
    }
    fn get_program_config_pda(&self) -> Pubkey {
        return self.program_config_pda;
    }
    fn get_treasury(&self) -> Pubkey {
        return self.treasury;
    }
    fn get_create_keypair(&self) -> &Option<Keypair>{
        return &self.multisig_create_keypair;
    }

    async fn new(args: BaseMultisigCreateArgs) -> Result<Self, Self::Error>
    {
        let program_id = squads_multisig_program::ID;

        let (multisig_pda, _)       = get_multisig_pda(&args.multisig_create_keypair.pubkey(), Some(&program_id));
        let (vault_pda, _)          = get_vault_pda(&multisig_pda, 0, Some(&program_id));
        let (program_config_pda, _) = get_program_config_pda(Some(&program_id));

        let program_config =  match args.rpc_client.get_account(&program_config_pda).await {
            Ok(account) => account,
            Err(_) => return Err(Self::Error::FailedToFetchProgramConfigAccount)
        };

        let mut program_config_data = program_config.data.as_slice();

        let treasury =
        match ProgramConfig::try_deserialize(&mut program_config_data) {
            Ok(config) => config,
            Err(_) => return Err(Self::Error::FailedToDeserializeProgramConfigData)
        }
        .treasury;

        Ok(BaseMultisig {
            rpc_client: args.rpc_client,
            multisig_create_keypair: Some(args.multisig_create_keypair),
            creator: args.creator,
            multisig_pda,
            vault_pda,
            program_config_pda,
            treasury
        })
    }

    async fn from_multisig_pda(args: BaseMultisigInitArgs) -> Result<Self, Self::Error> {
        let program_id = squads_multisig_program::ID;

        let multisig_pda     = args.multisig_pda;
        let (vault_pda, _)          = get_vault_pda(&multisig_pda, 0, Some(&program_id));
        let (program_config_pda, _) = get_program_config_pda(Some(&program_id));

        let program_config =  match args.rpc_client.get_account(&program_config_pda).await {
            Ok(account) => account,
            Err(_) => return Err(Self::Error::FailedToFetchProgramConfigAccount)
        };

        let mut program_config_data = program_config.data.as_slice();

        let treasury =
        match ProgramConfig::try_deserialize(&mut program_config_data) {
            Ok(config) => config,
            Err(_) => return Err(Self::Error::FailedToDeserializeProgramConfigData)
        }
        .treasury;

        Ok(BaseMultisig {
            rpc_client: args.rpc_client,
            multisig_create_keypair: None,
            creator: args.creator,
            multisig_pda,
            vault_pda,
            program_config_pda,
            treasury
        })
    }

    async fn get_multisig(&self) -> Result<Multisig, Self::Error>{
        let multisig_config =
        match self.rpc_client.get_account(&self.multisig_pda).await{
            Ok(account) => account,
            Err(_) => return Err(Self::Error::FailedToFetchMultisigConfigAccount)
        };

        let mut multisig_config_data = multisig_config.data.as_slice();
        let multisig =
        match Multisig::try_deserialize(&mut multisig_config_data) {
            Ok(a) => a,
            Err(_) => return Err(Self::Error::FailedToDeserializeMultisigConfigData)
        };

        Ok(multisig)
    }

    async fn get_current_proposal_status(&self) -> Result<ProposalStatus, Self::Error>{
        let program_id = squads_multisig_program::ID;
        let transaction_index = self.get_multisig_transaction_index().await?;
        let (proposal_pda, _) = get_proposal_pda(&self.multisig_pda, transaction_index, Some(&program_id));

        let proposal_config =
        match self.rpc_client.get_account(&proposal_pda).await{
            Ok(account) => account,
            Err(_) => return Err(Self::Error::FailedToFetchProposalConfigAccount)
        };

        let mut proposal_config_data = proposal_config.data.as_slice();
        let proposal =
        match Proposal::try_deserialize(&mut proposal_config_data) {
            Ok(a) => a,
            Err(_) => return Err(Self::Error::FailedToDeserializeProposalConfigData)
        };

        Ok(proposal.status)
    }

    async fn get_transaction_from_instructions(&self, sender: Pubkey, instructions: &[Instruction]) -> Result<Transaction, Self::Error> {
        let mut message = Message::new(instructions, Some(&sender));
        let recent_blockhash =
            match self.rpc_client.get_latest_blockhash().await {
                Ok(hash) => hash,
                Err(_) => return Err(Self::Error::ErrorOnGettingLatestBlockHash)
            };
        message.recent_blockhash = recent_blockhash;

        Ok(Transaction::new_unsigned(message))
    }

    async fn instruction_proposal_approve(&self, approver: Pubkey)  -> Result<Instruction, Self::Error> {
        let program_id: Pubkey = squads_multisig_program::ID;
        let transaction_index = self.get_multisig_transaction_index().await?;
        let (proposal_pda, _) = get_proposal_pda(&self.multisig_pda, transaction_index, Some(&program_id));

        let proposal_approve_ix = proposal_approve(
            ProposalVoteAccounts {
                multisig: self.multisig_pda,
                member: approver,
                proposal: proposal_pda
            },
            ProposalVoteArgs { memo: None },
            Some(program_id)
        );

        Ok(proposal_approve_ix)
    }

    async fn instruction_proposal_cancel(&self, canceler: Pubkey) -> Result<Instruction, Self::Error> {
        let program_id: Pubkey = squads_multisig_program::ID;
        let transaction_index = self.get_multisig_transaction_index().await?;
        let (proposal_pda, _) = get_proposal_pda(&self.multisig_pda, transaction_index, Some(&program_id));

        let proposal_status = self.get_current_proposal_status().await.unwrap();

        match proposal_status {
            ProposalStatus::Approved { timestamp: _ } => {},
            _ => return Err(Self::Error::ProposalStatusIsNotApproved)
        }

        let proposal_cancel_ix = proposal_cancel(
            ProposalVoteAccounts {
                multisig: self.multisig_pda,
                member: canceler,
                proposal: proposal_pda
            },
            ProposalVoteArgs { memo: None },
            Some(program_id)
        );

        Ok(proposal_cancel_ix)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::*;
    use solana_sdk::signature::Keypair;
    use tokio;

    #[tokio::test]
    async fn get_base_multisig_instance() -> Result<(), Box<dyn Error>> {
        let rpc_client = RpcClient::new("http://127.0.0.1:8899".to_string());
        let creator: Keypair = Keypair::new();
        let create_key = Keypair::new();

        let _ = BaseMultisig::new(BaseMultisigCreateArgs {
            rpc_client: RpcClient::new(rpc_client.url()),
            multisig_create_keypair: create_key.insecure_clone(),
            creator: creator.pubkey().clone()
        }).await?;

        Ok(())
    }
}