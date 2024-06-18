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
        let transaction_index = self.get_multisig_transaction_index().await?;
        let (proposal_pda, _) = get_proposal_pda(&self.multisig_pda, transaction_index, Some(&program_id));

        let proposal_status = self.get_current_proposal_status().await.unwrap();

        match proposal_status {
            ProposalStatus::Approved { timestamp: _ } => {},
            _ => return Err(InvestorMultisigError::ProposalStatusIsNotApproved)
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

    pub async fn transaction_proposal_approve(&self, approver: Pubkey)  -> Result<Transaction, InvestorMultisigError> {
        let ix = self.instruction_proposal_approve(approver).await?;

        Ok(self.get_transaction_from_instructions(approver, &[ix]).await?)

    }

    pub async fn transaction_proposal_cancel(&self, canceler: Pubkey) -> Result<Transaction, InvestorMultisigError> {
        let ix = self.instruction_proposal_cancel(canceler).await?;

        Ok(self.get_transaction_from_instructions(canceler, &[ix]).await?)
    }
}


#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::dao_module::{business_analyst_multisig::{BusinessAnalystMultisig, BusinessAnalystMultisigCreateArgs}, error::BusinessAnalystMultisigError};

    use super::*;
    use solana_sdk::{native_token::LAMPORTS_PER_SOL, signature::Signature};
    use squads_multisig_program::{Member, Permission, Permissions};
    use tokio;

    async fn transaction_sign_and_send(tx: &mut Transaction, keys: &[&Keypair], multisig_rpc: &RpcClient) -> Result<(), Box<dyn Error>> {
        let recent_blockhash = multisig_rpc.get_latest_blockhash().await.unwrap();
        let _ = tx.try_sign(keys, recent_blockhash);
        let _ = multisig_rpc.send_and_confirm_transaction(tx).await?;
        Ok(())
    }

    async fn get_ba_multisig(rpc_client: &RpcClient, multisig_create_keypair: &Keypair, creator: &Keypair, members: &[Member]) -> Result<BusinessAnalystMultisig, BusinessAnalystMultisigError> {
        let result = BusinessAnalystMultisig::new(BusinessAnalystMultisigCreateArgs {
            rpc_client: RpcClient::new(rpc_client.url()),
            multisig_create_keypair: multisig_create_keypair.insecure_clone(),
            creator: creator.pubkey().clone()
        }).await?;

        let mut tx = result.transaction_create_multisig(members, 1, 0).await?;
        let _ = transaction_sign_and_send(&mut tx, &[&creator, &multisig_create_keypair], rpc_client).await.unwrap();

        Ok(result)
    }

    async fn get_investor_multisig(multisig: &BusinessAnalystMultisig) -> Result<InvestorsMultisig, InvestorMultisigError> {
        let result = InvestorsMultisig::new(InvestorMultisigCreateArgs {
            rpc_client: RpcClient::new(multisig.rpc_client.url()),
            multisig_create_keypair: multisig.multisig_create_keypair.insecure_clone(),
            creator: multisig.creator.clone()
        }).await?;

        Ok(result)
    }

    pub async fn airdrop(rpc_client: &RpcClient, address: &Pubkey, amount: u64) -> Result<Signature, Box<dyn Error>> {
        let sig = rpc_client.request_airdrop(&address, (amount * LAMPORTS_PER_SOL) as u64).await?;
        println!("🚀Airdropping {} SOL to {} with sig {}",amount, address, sig );
        loop {
            let confirmed = rpc_client.confirm_transaction(&sig).await?;
            if confirmed {
                break;
            }
        }
        Ok(sig)
    }

    #[tokio::test]
    async fn create_multisig_with_investor() -> Result<(), Box<dyn Error>> {
        let rpc_client = RpcClient::new("http://127.0.0.1:8899".to_string());
        let ba: Keypair = Keypair::new();
        let investor_key: Keypair = Keypair::new();
        let create_key = Keypair::new();

        let investor = Member {
            key: investor_key.pubkey(),
            permissions: Permissions::from_vec(&[Permission::Vote]),
        };

        let _ = airdrop(&rpc_client, &ba.pubkey(), 1).await?;
        let multisig = get_ba_multisig(&rpc_client, &create_key, &ba, &[investor]).await.unwrap();
        let investor_multisig = get_investor_multisig(&multisig).await.unwrap();

        assert!(investor_multisig.is_member(investor_key.pubkey()).await.unwrap());
        assert_eq!(2, investor_multisig.get_multisig_members().await.unwrap().len());
        Ok(())
    }

    #[tokio::test]
    async fn approve_proposal() -> Result<(), Box<dyn Error>> {
        let rpc_client = RpcClient::new("http://127.0.0.1:8899".to_string());
        let ba: Keypair = Keypair::new();
        let investor_key: Keypair = Keypair::new();
        let create_key = Keypair::new();

        let investor = Member {
            key: investor_key.pubkey(),
            permissions: Permissions::from_vec(&[Permission::Vote]),
        };

        let _ = airdrop(&rpc_client, &ba.pubkey(), 1).await?;
        let _ = airdrop(&rpc_client, &investor_key.pubkey(), 1).await?;
        let ba_multisig = get_ba_multisig(&rpc_client, &create_key, &ba, &[investor]).await.unwrap();
        let investor_multisig = get_investor_multisig(&ba_multisig).await.unwrap();

        let mut tx = ba_multisig.transaction_change_threshold(ba.pubkey(), 2).await.unwrap();
        transaction_sign_and_send(&mut tx, &[&ba], &rpc_client).await.unwrap();

        let mut tx = ba_multisig.transaction_proposal_create(ba.pubkey()).await.unwrap();
        transaction_sign_and_send(&mut tx, &[&ba], &rpc_client).await.unwrap();

        let mut tx = investor_multisig.transaction_proposal_approve(investor_key.pubkey()).await.unwrap();
        transaction_sign_and_send(&mut tx, &[&investor_key], &rpc_client).await.unwrap();

        let proposal_status = investor_multisig.get_current_proposal_status().await.unwrap();

        match proposal_status {
            ProposalStatus::Approved { timestamp: _ } => return Ok(()),
            _ => panic!("Proposal status not Approved")
        }

    }

    #[tokio::test]
    async fn proposal_cancel() -> Result<(), Box<dyn Error>> {
        let rpc_client = RpcClient::new("http://127.0.0.1:8899".to_string());
        let ba: Keypair = Keypair::new();
        let investor_key: Keypair = Keypair::new();
        let create_key = Keypair::new();

        let investor = Member {
            key: investor_key.pubkey(),
            permissions: Permissions::from_vec(&[Permission::Vote]),
        };

        let _ = airdrop(&rpc_client, &ba.pubkey(), 1).await?;
        let _ = airdrop(&rpc_client, &investor_key.pubkey(), 1).await?;

        let ba_multisig = get_ba_multisig(&rpc_client, &create_key, &ba, &[investor]).await.unwrap();
        let investor_multisig = get_investor_multisig(&ba_multisig).await.unwrap();

        let mut tx = ba_multisig.transaction_change_threshold(ba.pubkey(), 2).await.unwrap();
        transaction_sign_and_send(&mut tx, &[&ba], &rpc_client).await.unwrap();

        let mut tx = ba_multisig.transaction_proposal_create(ba.pubkey()).await.unwrap();
        transaction_sign_and_send(&mut tx, &[&ba], &rpc_client).await.unwrap();

        let mut tx = investor_multisig.transaction_proposal_approve(ba.pubkey()).await.unwrap();
        transaction_sign_and_send(&mut tx, &[&ba], &rpc_client).await.unwrap();

        let mut tx = investor_multisig.transaction_proposal_cancel(investor_key.pubkey()).await.unwrap();
        transaction_sign_and_send(&mut tx, &[&investor_key], &rpc_client).await.unwrap();

        let proposal_status = investor_multisig.get_current_proposal_status().await.unwrap();

        match proposal_status {
            ProposalStatus::Cancelled { timestamp: _ } => return Ok(()),
            _ => panic!("Proposal status not Cancelled")
        }
    }
}