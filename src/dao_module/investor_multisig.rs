use async_trait::async_trait;
use solana_sdk::{
    instruction::Instruction,
    message::Message,
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transaction::Transaction
};
use squads_multisig::{
    anchor_lang::AccountDeserialize,
    client::{
        proposal_approve,
        proposal_cancel,
        ProposalVoteAccounts,
        ProposalVoteArgs
    },
    pda::get_proposal_pda,
    solana_client::nonblocking::rpc_client::RpcClient,
    squads_multisig_program::{
        self,
        state::ProgramConfig,
        Multisig
    },
    state::{
        Proposal,
        ProposalStatus
    }
};
use super::{base_multisig::BaseMultisig, error::InvestorMultisigError};

pub struct InvestorMultisigCreateArgs {
    pub rpc_client: RpcClient,
    pub multisig_create_keypair: Keypair,
    pub creator: Pubkey
}
pub struct InvestorsMultisig {
    pub rpc_client: RpcClient,
    multisig_create_keypair: Keypair,
    pub creator: Pubkey,
    pub multisig_pda: Pubkey,
    pub vault_pda: Pubkey,
    pub program_config_pda: Pubkey,
    pub treasury: Pubkey
}

#[async_trait]
impl BaseMultisig<InvestorMultisigCreateArgs> for InvestorsMultisig {
    type Error = InvestorMultisigError;

    async fn new(create_args: InvestorMultisigCreateArgs) -> Result<Self, Self::Error> {
        let program_id = squads_multisig_program::ID;

        let (multisig_pda, _) = squads_multisig::pda::get_multisig_pda(&create_args.multisig_create_keypair.pubkey(), Some(&program_id));
        let (vault_pda, _) = squads_multisig::pda::get_vault_pda(&multisig_pda, 0, Some(&program_id));
        let (program_config_pda, _) = squads_multisig::pda::get_program_config_pda(Some(&program_id));

        let program_config =  match create_args.rpc_client.get_account(&program_config_pda).await {
            Ok(account) => account,
            Err(_) => return Err(InvestorMultisigError::FailedToFetchProgramConfigAccount)
        };

        let mut program_config_data = program_config.data.as_slice();

        let treasury =
        match ProgramConfig::try_deserialize(&mut program_config_data) {
            Ok(config) => config,
            Err(_) => return Err(InvestorMultisigError::FailedToDeserializeProgramConfigData)
        }
        .treasury;

        Ok(InvestorsMultisig {
            rpc_client: create_args.rpc_client,
            multisig_create_keypair: create_args.multisig_create_keypair,
            creator: create_args.creator,
            multisig_pda,
            vault_pda,
            program_config_pda,
            treasury
        })
    }

    fn get_multisig_create_args(&self) -> InvestorMultisigCreateArgs {
        InvestorMultisigCreateArgs {
            rpc_client: RpcClient::new(self.rpc_client.url()),
            multisig_create_keypair: self.multisig_create_keypair.insecure_clone(),
            creator: self.creator.clone()
        }
    }

    async fn get_multisig(&self) -> Result<Multisig, Self::Error> {
        let multisig_config =
        match self.rpc_client.get_account(&self.multisig_pda).await{
            Ok(account) => account,
            Err(_) => return Err(InvestorMultisigError::FailedToFetchMultisigConfigAccount)
        };

        let mut multisig_config_data = multisig_config.data.as_slice();
        let multisig =
        match Multisig::try_deserialize(&mut multisig_config_data) {
            Ok(a) => a,
            Err(_) => return Err(InvestorMultisigError::FailedToDeserializeMultisigConfigData)
        };

        Ok(multisig)
    }

    async fn get_current_proposal_status(&self) -> Result<ProposalStatus, Self::Error> {
        let program_id = squads_multisig_program::ID;
        let transaction_index = self.get_multisig_transaction_index().await?;
        let (proposal_pda, _) = get_proposal_pda(&self.multisig_pda, transaction_index, Some(&program_id));

        let proposal_config =
        match self.rpc_client.get_account(&proposal_pda).await{
            Ok(account) => account,
            Err(_) => return Err(InvestorMultisigError::FailedToFetchProposalConfigAccount)
        };

        let mut proposal_config_data = proposal_config.data.as_slice();
        let proposal =
        match Proposal::try_deserialize(&mut proposal_config_data) {
            Ok(a) => a,
            Err(_) => return Err(InvestorMultisigError::FailedToDeserializeProposalConfigData)
        };

        Ok(proposal.status)
    }

    async fn get_transaction_from_instructions(&self, sender: Pubkey, instructions: &[Instruction]) -> Result<Transaction, Self::Error> {
        let mut message = Message::new(instructions, Some(&sender));
        let recent_blockhash =
            match self.rpc_client.get_latest_blockhash().await {
                Ok(hash) => hash,
                Err(_) => return Err(InvestorMultisigError::ErrorOnGettingLatestBlockHash)
            };
        message.recent_blockhash = recent_blockhash;

        Ok(Transaction::new_unsigned(message))
    }
}

impl InvestorsMultisig {
    pub async fn instruction_proposal_approve(&self, approver: Pubkey)  -> Result<Instruction, InvestorMultisigError> {
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

    pub async fn instruction_proposal_cancel(&self, canceler: Pubkey) -> Result<Instruction, InvestorMultisigError> {
        let program_id: Pubkey = squads_multisig_program::ID;
        let transaction_index = self.get_multisig_transaction_index().await? + 1;
        let (proposal_pda, _) = get_proposal_pda(&self.multisig_pda, transaction_index, Some(&program_id));

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

    pub async fn transaction_proposal_approve(&self, approver: Pubkey)  -> Result<Transaction, InvestorMultisigError> {
        let ix = self.instruction_proposal_approve(approver).await?;

        Ok(self.get_transaction_from_instructions(approver, &[ix]).await?)

    }

    pub async fn transaction_proposal_cancel(&self, canceler: Pubkey) -> Result<Transaction, InvestorMultisigError> {
        let ix = self.instruction_proposal_cancel(canceler).await?;

        Ok(self.get_transaction_from_instructions(canceler, &[ix]).await?)
    }
}