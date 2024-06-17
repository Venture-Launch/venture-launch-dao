use solana_sdk::{
        instruction::Instruction,
        message::Message,
        pubkey::Pubkey,
        signature::Keypair,
        signer::Signer,
        system_program,
        transaction::Transaction
    };
use squads_multisig::{
    anchor_lang::AccountDeserialize,
    client::{
        self,
        config_transaction_create,
        config_transaction_execute,
        multisig_create_v2,
        proposal_approve,
        proposal_cancel,
        proposal_create,
        ConfigTransactionCreateAccounts,
        ConfigTransactionCreateArgs,
        ConfigTransactionExecuteAccounts,
        MultisigCreateAccountsV2,
        MultisigCreateArgsV2,
        ProposalCreateArgs,
        ProposalVoteAccounts,
        ProposalVoteArgs
    },
    pda::{
        get_proposal_pda,
        get_transaction_pda
    },
    solana_client::nonblocking::rpc_client::RpcClient,
    squads_multisig_program::{
        self,
        state::ProgramConfig,
        Multisig
    },
    state::{
        ConfigAction,
        Member,
        Permission,
        Permissions,
        Proposal,
        ProposalStatus
    }
};
use super::error::MultisigError;

pub struct InvestorMultisigCreateArgs {
    pub rpc_client: RpcClient,
    pub multisig_create_keypair: Keypair,
    pub creator: Pubkey
}
pub struct InvestorsMultisig {
    pub create_args: InvestorMultisigCreateArgs,
    pub multisig_pda: Pubkey,
    pub vault_pda: Pubkey,
    pub program_config_pda: Pubkey,
    pub treasury: Pubkey
}

impl InvestorsMultisig {
    pub async fn new(create_args: InvestorMultisigCreateArgs) -> Result<Self, MultisigError> {
        let program_id = squads_multisig_program::ID;

        let (multisig_pda, _) = squads_multisig::pda::get_multisig_pda(&create_args.multisig_create_keypair.pubkey(), Some(&program_id));
        let (vault_pda, _) = squads_multisig::pda::get_vault_pda(&multisig_pda, 0, Some(&program_id));
        let (program_config_pda, _) = squads_multisig::pda::get_program_config_pda(Some(&program_id));

        let program_config =  match create_args.rpc_client.get_account(&program_config_pda).await {
            Ok(account) => account,
            Err(_) => return Err(MultisigError::FailedToFetchProgramConfigAccount)
        };

        let mut program_config_data = program_config.data.as_slice();

        let treasury =
        match ProgramConfig::try_deserialize(&mut program_config_data) {
            Ok(config) => config,
            Err(_) => return Err(MultisigError::FailedToDeserializeProgramConfigData)
        }
        .treasury;

        Ok(InvestorsMultisig {
            create_args,
            multisig_pda,
            vault_pda,
            program_config_pda,
            treasury
        })
    }

    pub async fn get_multisig(&self) -> Result<Multisig, MultisigError> {
        let multisig_config =
        match self.create_args.rpc_client.get_account(&self.multisig_pda).await{
            Ok(account) => account,
            Err(_) => return Err(MultisigError::FailedToFetchMultisigConfigAccount)
        };

        let mut multisig_config_data = multisig_config.data.as_slice();
        let multisig =
        match Multisig::try_deserialize(&mut multisig_config_data) {
            Ok(a) => a,
            Err(_) => return Err(MultisigError::FailedToDeserializeMultisigConfigData)
        };

        Ok(multisig)
    }

    pub async fn get_multisig_transaction_index(&self) -> Result<u64, MultisigError> {
        let multisig = self.get_multisig().await?;

        Ok(multisig.transaction_index)
    }

    pub async fn get_multisig_members(&self) -> Result<Vec<Member>, MultisigError> {
        let multisig = self.get_multisig().await?;

        Ok(multisig.members)
    }

    pub async fn get_threshold(&self) -> Result<u16, MultisigError> {
        let multisig = self.get_multisig().await?;

        Ok(multisig.threshold)
    }

    pub async fn is_member(&self, member_pubkey: Pubkey) -> Result<bool, MultisigError> {
        let multisig = self.get_multisig().await?;

        Ok(multisig.is_member(member_pubkey).is_some())
    }

    pub async fn get_current_proposal_status(&self) -> Result<ProposalStatus, MultisigError> {
        let program_id = squads_multisig_program::ID;
        let transaction_index = self.get_multisig_transaction_index().await?;
        let (proposal_pda, _) = get_proposal_pda(&self.multisig_pda, transaction_index, Some(&program_id));

        let proposal_config =
        match self.create_args.rpc_client.get_account(&proposal_pda).await{
            Ok(account) => account,
            Err(_) => return Err(MultisigError::FailedToFetchProposalConfigAccount)
        };

        let mut proposal_config_data = proposal_config.data.as_slice();
        let proposal =
        match Proposal::try_deserialize(&mut proposal_config_data) {
            Ok(a) => a,
            Err(_) => return Err(MultisigError::FailedToDeserializeProposalConfigData)
        };

        Ok(proposal.status)
    }

    async fn get_transaction_from_instructions(&self, sender: Pubkey, instructions: &[Instruction]) -> Result<Transaction, MultisigError> {
        let mut message = Message::new(instructions, Some(&sender));
        let recent_blockhash =
            match self.create_args.rpc_client.get_latest_blockhash().await {
                Ok(hash) => hash,
                Err(_) => return Err(MultisigError::ErrorOnGettingLatestBlockHash)
            };
        message.recent_blockhash = recent_blockhash;

        Ok(Transaction::new_unsigned(message))
    }

    pub fn instruction_create_multisig(&self, members: &[Member], threshold: u16, time_lock: u32) -> Instruction {

        let mut members: Vec<Member> = members.to_vec();
        let creator = Member {
            key: self.create_args.creator,
            permissions: Permissions::from_vec(&[Permission::Initiate, Permission::Vote, Permission::Execute]),
        };

        if !members.contains(&creator) {
            members.push(creator);
        }

        multisig_create_v2(
            MultisigCreateAccountsV2 {
                program_config: self.program_config_pda,
                treasury: self.treasury,
                multisig: self.multisig_pda,
                create_key: self.create_args.multisig_create_keypair.pubkey(),
                creator: self.create_args.creator,
                system_program: system_program::ID,
            },
            MultisigCreateArgsV2 {
                members,
                threshold,
                time_lock,
                config_authority: None,
                rent_collector: None,
                memo: Some("Deploy my own Squad".to_string()),
            },
            Some(squads_multisig_program::ID)
        )
    }

    pub async fn transaction_create_multisig(&self, members: &[Member], threshold: u16, time_lock: u32) -> Result<Transaction, MultisigError> {
        let instruction = self.instruction_create_multisig(members, threshold, time_lock);

        Ok(self.get_transaction_from_instructions(self.create_args.creator, &[instruction]).await?)
    }

    /// Creates a new config_transaction instruction to add member on behalf of adder.
    pub async fn instructions_add_member(&self, adder: Pubkey, new_member: Member) -> Result<Instruction, MultisigError> {
        let program_id: Pubkey = squads_multisig_program::ID;
        let transaction_index = self.get_multisig_transaction_index().await? + 1;
        let (transaction_pda, _) = get_transaction_pda(&self.multisig_pda, transaction_index, Some(&program_id));

        let add_member_ix = config_transaction_create(
            ConfigTransactionCreateAccounts {
                multisig: self.multisig_pda,
                transaction: transaction_pda,
                creator: adder,
                rent_payer: adder,
                system_program: system_program::ID
            }
            , ConfigTransactionCreateArgs{
                memo: Some(format!("Add {} as member to multisig {}", new_member.key.to_string(), self.multisig_pda)),
                actions: vec![ConfigAction::AddMember {new_member: new_member}]
            },
            Some(program_id)
        );

        Ok(add_member_ix)
    }

    pub async fn instructions_remove_member(&self, remover: Pubkey, old_member_pubkey: Pubkey) -> Result<Instruction, MultisigError> {
        let program_id: Pubkey = squads_multisig_program::ID;
        let transaction_index = self.get_multisig_transaction_index().await? + 1;
        let (transaction_pda, _) = get_transaction_pda(&self.multisig_pda, transaction_index, Some(&program_id));

        let remove_member_ix = config_transaction_create(
            ConfigTransactionCreateAccounts {
                multisig: self.multisig_pda,
                transaction: transaction_pda,
                creator: remover,
                rent_payer: remover,
                system_program: system_program::ID
            }
            , ConfigTransactionCreateArgs{
                memo: Some(format!("Remove {} member from multisig {}", old_member_pubkey.to_string(), self.multisig_pda)),
                actions: vec![ConfigAction::RemoveMember { old_member: old_member_pubkey }]
            },
            Some(program_id)
        );

        Ok(remove_member_ix)
    }

    pub async fn instruction_proposal_create(&self, creator: Pubkey)  -> Result<Instruction, MultisigError> {
        let program_id: Pubkey = squads_multisig_program::ID;
        let transaction_index = self.get_multisig_transaction_index().await?;
        let (proposal_pda, _) = get_proposal_pda(&self.multisig_pda, transaction_index, Some(&program_id));

        let proposal_create_ix = proposal_create(
            client::ProposalCreateAccounts {
                multisig: self.multisig_pda,
                proposal: proposal_pda,
                creator: creator,
                rent_payer: creator,
                system_program: system_program::ID
            }
            , ProposalCreateArgs {
                transaction_index,
                draft: false
            },
            Some(program_id)
        );

        Ok(proposal_create_ix)
    }

    pub async fn instruction_proposal_approve(&self, approver: Pubkey)  -> Result<Instruction, MultisigError> {
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

    pub async fn instruction_proposal_cancel(&self, canceler: Pubkey) -> Result<Instruction, MultisigError> {
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

    pub async fn instruction_config_transaction_execute(&self, executer: Pubkey) -> Result<Instruction, MultisigError> {
        let program_id: Pubkey = squads_multisig_program::ID;
        let transaction_index = self.get_multisig_transaction_index().await?;
        let (proposal_pda, _) = get_proposal_pda(&self.multisig_pda, transaction_index, Some(&program_id));
        let (transaction_pda, _) = get_transaction_pda(&self.multisig_pda, transaction_index, Some(&program_id));

        let config_transaction_execute_ix = config_transaction_execute(
            ConfigTransactionExecuteAccounts {
                multisig: self.multisig_pda,
                member: executer,
                proposal: proposal_pda,
                transaction: transaction_pda,
                rent_payer: Some(executer),
                system_program: Some(system_program::ID),
            },
            vec![],
            Some(program_id)
        );

        Ok(config_transaction_execute_ix)
    }

     /// Creates a new config transaction to add member on behalf of adder.
    pub async fn transaction_add_member(&self, adder: Pubkey, new_member: Member) -> Result<Transaction, MultisigError> {
        let ix = self.instructions_add_member(adder, new_member).await?;

        Ok(self.get_transaction_from_instructions(adder, &[ix]).await?)
    }

    pub async fn transaction_remove_member(&self, remover: Pubkey, old_member_pubkey: Pubkey) -> Result<Transaction, MultisigError> {
        let ix = self.instructions_remove_member(remover, old_member_pubkey).await?;

        Ok(self.get_transaction_from_instructions(remover, &[ix]).await?)
    }

    pub async fn transaction_proposal_create(&self, creator: Pubkey)  -> Result<Transaction, MultisigError> {
        let ix = self.instruction_proposal_create(creator).await?;

        Ok(self.get_transaction_from_instructions(creator, &[ix]).await?)

    }

    pub async fn transaction_proposal_approve(&self, approver: Pubkey)  -> Result<Transaction, MultisigError> {
        let ix = self.instruction_proposal_approve(approver).await?;

        Ok(self.get_transaction_from_instructions(approver, &[ix]).await?)

    }

    pub async fn transaction_proposal_cancel(&self, canceler: Pubkey) -> Result<Transaction, MultisigError> {
        let ix = self.instruction_proposal_cancel(canceler).await?;

        Ok(self.get_transaction_from_instructions(canceler, &[ix]).await?)
    }

    pub async fn transaction_config_transaction_execute(&self, executer: Pubkey) -> Result<Transaction, MultisigError> {
        let ix = self.instruction_config_transaction_execute(executer).await?;

        Ok(self.get_transaction_from_instructions(executer, &[ix]).await?)
    }

    pub async fn instruction_change_threshold(&self, changer: Pubkey, new_threshold: u16) -> Result<Instruction, MultisigError> {
        let program_id: Pubkey = squads_multisig_program::ID;
        let transaction_index = self.get_multisig_transaction_index().await? + 1;
        let (transaction_pda, _) = get_transaction_pda(&self.multisig_pda, transaction_index, Some(&program_id));

        let change_threshold_ix = config_transaction_create(
            ConfigTransactionCreateAccounts {
                multisig: self.multisig_pda,
                transaction: transaction_pda,
                creator: changer,
                rent_payer: changer,
                system_program: system_program::ID
            }
            , ConfigTransactionCreateArgs{
                memo: Some(format!("Changing threshold to {} on multisig {}", new_threshold, self.multisig_pda)),
                actions: vec![ConfigAction::ChangeThreshold { new_threshold }]
            },
            Some(program_id)
        );

        Ok(change_threshold_ix)
    }

    pub async fn transaction_change_threshold(&self, changer: Pubkey, new_threshold: u16) -> Result<Transaction, MultisigError> {
        let ix = self.instruction_change_threshold(changer, new_threshold).await?;

        Ok(self.get_transaction_from_instructions(changer, &[ix]).await?)
    }
}