use std::str::FromStr;

use super::{
    base_multisig::{BaseMultisig, BaseMultisigCreateArgs},
    base_multisig_trait::BaseMultisigTrait,
    error::BaseMultisigError,
};
use async_trait::async_trait;
use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer, system_instruction, system_program, transaction::Transaction
};
use spl_associated_token_account::get_associated_token_address_with_program_id;
use squads_multisig::{
    client::{
        self, config_transaction_create, config_transaction_execute, multisig_create_v2,
        proposal_create, vault_transaction_create, vault_transaction_execute,
        ConfigTransactionCreateAccounts, ConfigTransactionCreateArgs,
        ConfigTransactionExecuteAccounts, MultisigCreateAccountsV2, MultisigCreateArgsV2,
        ProposalCreateArgs, VaultTransactionCreateAccounts, VaultTransactionExecuteAccounts,
    },
    pda::{get_proposal_pda, get_transaction_pda},
    squads_multisig_program,
    state::{ConfigAction, Member, Permission, Permissions, TransactionMessage},
    vault_transaction::VaultTransactionMessageExt,
};

#[async_trait]
pub trait BusinessAnalystMultisigTrait<Args = BaseMultisigCreateArgs>:
    BaseMultisigTrait<Args, Error = BaseMultisigError>
{
    fn instruction_create_multisig(
        &self,
        members: &[Member],
        threshold: u16,
        time_lock: u32,
        multisig_create_keypair: &Keypair
    ) -> Instruction;
    async fn transaction_create_multisig(
        &self,
        members: &[Member],
        threshold: u16,
        time_lock: u32,
        multisig_create_keypair: &Keypair
    ) -> Result<Transaction, Self::Error>;

    async fn instructions_add_member(
        &self,
        adder: Pubkey,
        new_member: Member,
    ) -> Result<Instruction, Self::Error>;
    async fn instructions_remove_member(
        &self,
        remover: Pubkey,
        old_member_pubkey: Pubkey,
    ) -> Result<Instruction, Self::Error>;
    async fn instruction_transfer_from_vault(
        &self,
        sender: Pubkey,
        receiver: Pubkey,
        lamports: u64,
    ) -> Result<Instruction, Self::Error>;
    async fn instruction_proposal_create(
        &self,
        creator: Pubkey,
    ) -> Result<Instruction, Self::Error>;
    async fn instruction_config_transaction_execute(
        &self,
        executer: Pubkey,
    ) -> Result<Instruction, Self::Error>;
    async fn instruction_vault_transaction_execute(
        &self,
        sender: Pubkey,
        receiver: Pubkey,
        lamports: u64,
    ) -> Result<Instruction, Self::Error>;
    async fn instruction_change_threshold(
        &self,
        changer: Pubkey,
        new_threshold: u16,
    ) -> Result<Instruction, Self::Error>;

    async fn transaction_add_member(
        &self,
        adder: Pubkey,
        new_member: Member,
    ) -> Result<Transaction, Self::Error> {
        let ix = self.instructions_add_member(adder, new_member).await?;

        Ok(self.get_transaction_from_instructions(adder, &[ix]).await?)
    }
    async fn transaction_remove_member(
        &self,
        remover: Pubkey,
        old_member_pubkey: Pubkey,
    ) -> Result<Transaction, Self::Error> {
        let ix = self
            .instructions_remove_member(remover, old_member_pubkey)
            .await?;

        Ok(self
            .get_transaction_from_instructions(remover, &[ix])
            .await?)
    }
    async fn transaction_transfer_from_vault(
        &self,
        sender: Pubkey,
        receiver: Pubkey,
        lamports: u64,
    ) -> Result<Transaction, Self::Error> {
        let ix = self
            .instruction_transfer_from_vault(sender, receiver, lamports)
            .await?;

        Ok(self
            .get_transaction_from_instructions(sender, &[ix])
            .await?)
    }
    async fn transaction_proposal_create(
        &self,
        creator: Pubkey,
    ) -> Result<Transaction, Self::Error> {
        let ix = self.instruction_proposal_create(creator).await?;

        Ok(self
            .get_transaction_from_instructions(creator, &[ix])
            .await?)
    }
    async fn transaction_config_transaction_execute(
        &self,
        executer: Pubkey,
    ) -> Result<Transaction, Self::Error> {
        let ix = self
            .instruction_config_transaction_execute(executer)
            .await?;

        Ok(self
            .get_transaction_from_instructions(executer, &[ix])
            .await?)
    }
    async fn transaction_vault_transaction_execute(
        &self,
        sender: Pubkey,
        receiver: Pubkey,
        lamports: u64,
    ) -> Result<Transaction, Self::Error> {
        let ix = self
            .instruction_vault_transaction_execute(sender, receiver, lamports)
            .await?;

        Ok(self
            .get_transaction_from_instructions(sender, &[ix])
            .await?)
    }

    async fn transaction_change_threshold(
        &self,
        changer: Pubkey,
        new_threshold: u16,
    ) -> Result<Transaction, Self::Error> {
        let ix = self
            .instruction_change_threshold(changer, new_threshold)
            .await?;

        Ok(self
            .get_transaction_from_instructions(changer, &[ix])
            .await?)
    }
}

#[async_trait]
impl BusinessAnalystMultisigTrait<BaseMultisigCreateArgs> for BaseMultisig {
    async fn transaction_create_multisig(
        &self,
        members: &[Member],
        threshold: u16,
        time_lock: u32,
        multisig_create_keypair: &Keypair
    ) -> Result<Transaction, Self::Error> {
        let instruction = self.instruction_create_multisig(members, threshold, time_lock, multisig_create_keypair);

        Ok(self
            .get_transaction_from_instructions(self.creator, &[instruction])
            .await?)
    }

    fn instruction_create_multisig(
        &self,
        members: &[Member],
        threshold: u16,
        time_lock: u32,
        multisig_create_keypair: &Keypair
    ) -> Instruction {
        let mut members: Vec<Member> = members.to_vec();
        let creator = Member {
            key: self.creator,
            permissions: Permissions::from_vec(&[
                Permission::Initiate,
                Permission::Vote,
                Permission::Execute,
            ]),
        };

        if !members.contains(&creator) {
            members.push(creator);
        }

        multisig_create_v2(
            MultisigCreateAccountsV2 {
                program_config: self.program_config_pda,
                treasury: self.treasury,
                multisig: self.multisig_pda,
                create_key: multisig_create_keypair.pubkey(),
                creator: self.creator,
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
            Some(squads_multisig_program::ID),
        )
    }

    async fn instructions_add_member(
        &self,
        adder: Pubkey,
        new_member: Member,
    ) -> Result<Instruction, Self::Error> {
        let program_id: Pubkey = squads_multisig_program::ID;
        let transaction_index = self.get_multisig_transaction_index().await? + 1;
        let (transaction_pda, _) =
            get_transaction_pda(&self.multisig_pda, transaction_index, Some(&program_id));

        let add_member_ix = config_transaction_create(
            ConfigTransactionCreateAccounts {
                multisig: self.multisig_pda,
                transaction: transaction_pda,
                creator: adder,
                rent_payer: adder,
                system_program: system_program::ID,
            },
            ConfigTransactionCreateArgs {
                memo: Some(format!(
                    "Add {} as member to multisig {}",
                    new_member.key.to_string(),
                    self.multisig_pda
                )),
                actions: vec![ConfigAction::AddMember {
                    new_member: new_member,
                }],
            },
            Some(program_id),
        );

        Ok(add_member_ix)
    }

    async fn instructions_remove_member(
        &self,
        remover: Pubkey,
        old_member_pubkey: Pubkey,
    ) -> Result<Instruction, Self::Error> {
        let program_id: Pubkey = squads_multisig_program::ID;
        let transaction_index = self.get_multisig_transaction_index().await? + 1;
        let (transaction_pda, _) =
            get_transaction_pda(&self.multisig_pda, transaction_index, Some(&program_id));

        let remove_member_ix = config_transaction_create(
            ConfigTransactionCreateAccounts {
                multisig: self.multisig_pda,
                transaction: transaction_pda,
                creator: remover,
                rent_payer: remover,
                system_program: system_program::ID,
            },
            ConfigTransactionCreateArgs {
                memo: Some(format!(
                    "Remove {} member from multisig {}",
                    old_member_pubkey.to_string(),
                    self.multisig_pda
                )),
                actions: vec![ConfigAction::RemoveMember {
                    old_member: old_member_pubkey,
                }],
            },
            Some(program_id),
        );

        Ok(remove_member_ix)
    }

    async fn instruction_transfer_from_vault(
        &self,
        sender: Pubkey,
        receiver: Pubkey,
        lamports: u64,
    ) -> Result<Instruction, Self::Error> {
        let program_id: Pubkey = squads_multisig_program::ID;
        let transaction_index = self.get_multisig_transaction_index().await? + 1;
        let (transaction_pda, _) =
            get_transaction_pda(&self.multisig_pda, transaction_index, Some(&program_id));
        let vault_index = 0;

        let token_program_id = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap();
        let token_mint = Pubkey::from_str("Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr").unwrap();

        let source_pubkey = get_associated_token_address_with_program_id(&self.vault_pda, &token_mint, &token_program_id);
        let destination_pubkey = get_associated_token_address_with_program_id(&receiver, &token_mint, &token_program_id);

        let ix = spl_token::instruction::transfer(
            &token_program_id,
            &source_pubkey,
            &destination_pubkey,
            &self.vault_pda,
            &[&self.vault_pda],
            lamports).unwrap();

        let message = TransactionMessage::try_compile(
            &self.vault_pda,
            &[ix],
            &[],
        )
        .unwrap();

        let transfer_from_vault_ix = vault_transaction_create(
            VaultTransactionCreateAccounts {
                multisig: self.multisig_pda,
                transaction: transaction_pda,
                creator: sender,
                rent_payer: sender,
                system_program: system_program::id(),
            },
            vault_index,
            0,
            &message,
            Some(format!(
                "Sending {lamports} lamports from {} to {}",
                self.vault_pda.to_string(),
                receiver.to_string()
            )),
            Some(program_id),
        );

        Ok(transfer_from_vault_ix)
    }

    async fn instruction_proposal_create(
        &self,
        creator: Pubkey,
    ) -> Result<Instruction, Self::Error> {
        let program_id: Pubkey = squads_multisig_program::ID;
        let transaction_index = self.get_multisig_transaction_index().await?;
        let (proposal_pda, _) =
            get_proposal_pda(&self.multisig_pda, transaction_index, Some(&program_id));

        let proposal_create_ix = proposal_create(
            client::ProposalCreateAccounts {
                multisig: self.multisig_pda,
                proposal: proposal_pda,
                creator: creator,
                rent_payer: creator,
                system_program: system_program::ID,
            },
            ProposalCreateArgs {
                transaction_index,
                draft: false,
            },
            Some(program_id),
        );

        Ok(proposal_create_ix)
    }

    async fn instruction_config_transaction_execute(
        &self,
        executer: Pubkey,
    ) -> Result<Instruction, Self::Error> {
        let program_id: Pubkey = squads_multisig_program::ID;
        let transaction_index = self.get_multisig_transaction_index().await?;
        let (proposal_pda, _) =
            get_proposal_pda(&self.multisig_pda, transaction_index, Some(&program_id));
        let (transaction_pda, _) =
            get_transaction_pda(&self.multisig_pda, transaction_index, Some(&program_id));

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
            Some(program_id),
        );

        Ok(config_transaction_execute_ix)
    }

    async fn instruction_vault_transaction_execute(
        &self,
        sender: Pubkey,
        receiver: Pubkey,
        lamports: u64,
    ) -> Result<Instruction, Self::Error> {
        let program_id: Pubkey = squads_multisig_program::ID;
        let transaction_index = self.get_multisig_transaction_index().await?;
        let (transaction_pda, _) =
            get_transaction_pda(&self.multisig_pda, transaction_index, Some(&program_id));
        let (proposal_pda, _) =
            get_proposal_pda(&self.multisig_pda, transaction_index, Some(&program_id));
        let vault_index = 0;

        let token_program_id = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap();
        let token_mint = Pubkey::from_str("Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr").unwrap();

        let source_pubkey = get_associated_token_address_with_program_id(&self.vault_pda, &token_mint, &token_program_id);
        let destination_pubkey = get_associated_token_address_with_program_id(&receiver, &token_mint, &token_program_id);

        println!("{destination_pubkey}");

        let ix = spl_token::instruction::transfer(
            &token_program_id,
            &source_pubkey,
            &destination_pubkey,
            &self.vault_pda,
            &[&self.vault_pda],
            lamports).unwrap();

        let message = TransactionMessage::try_compile(
            &self.vault_pda,
            &[ix],
            &[],
        )
        .unwrap();

        let vault_transaction_execute_ix = vault_transaction_execute(
            VaultTransactionExecuteAccounts {
                multisig: self.multisig_pda,
                transaction: transaction_pda,
                member: sender,
                proposal: proposal_pda,
            },
            vault_index,
            0,
            &message,
            &[],
            Some(program_id),
        );

        match vault_transaction_execute_ix {
            Ok(ix) => Ok(ix),
            Err(_) => Err(Self::Error::FailedToBuildVaultTransactionExecuteInstruction),
        }
    }

    async fn instruction_change_threshold(
        &self,
        changer: Pubkey,
        new_threshold: u16,
    ) -> Result<Instruction, Self::Error> {
        let program_id: Pubkey = squads_multisig_program::ID;
        let transaction_index = self.get_multisig_transaction_index().await? + 1;
        let (transaction_pda, _) =
            get_transaction_pda(&self.multisig_pda, transaction_index, Some(&program_id));

        let change_threshold_ix = config_transaction_create(
            ConfigTransactionCreateAccounts {
                multisig: self.multisig_pda,
                transaction: transaction_pda,
                creator: changer,
                rent_payer: changer,
                system_program: system_program::ID,
            },
            ConfigTransactionCreateArgs {
                memo: Some(format!(
                    "Changing threshold to {} on multisig {}",
                    new_threshold, self.multisig_pda
                )),
                actions: vec![ConfigAction::ChangeThreshold { new_threshold }],
            },
            Some(program_id),
        );

        Ok(change_threshold_ix)
    }
}

#[cfg(test)]
mod tests {
    use std::{error::Error, sync::Arc};

    use crate::multisig_utils::error::BaseMultisigError;

    use super::*;
    use solana_client::nonblocking::rpc_client::RpcClient;
    use solana_sdk::{
        native_token::LAMPORTS_PER_SOL,
        signature::{Keypair, Signature},
    };
    use tokio;

    async fn transaction_sign_and_send(
        tx: &mut Transaction,
        keys: &[&Keypair],
        multisig_rpc: &RpcClient,
    ) -> Result<(), Box<dyn Error>> {
        let recent_blockhash = multisig_rpc.get_latest_blockhash().await.unwrap();
        let _ = tx.try_sign(keys, recent_blockhash);
        let _ = multisig_rpc.send_and_confirm_transaction(tx).await?;
        Ok(())
    }

    async fn get_base_multisig(
        rpc_client: &RpcClient,
        multisig_create_keypair: &Keypair,
        creator: &Keypair,
        members: &[Member]
    ) -> Result<BaseMultisig, BaseMultisigError> {
        let result = BaseMultisig::new(BaseMultisigCreateArgs {
            rpc_client: RpcClient::new(rpc_client.url()),
            multisig_create_keypair: multisig_create_keypair.insecure_clone(),
            creator: creator.pubkey().clone(),
        })
        .await?;

        let mut tx = result.transaction_create_multisig(members, 1, 0, multisig_create_keypair).await?;
        let _ =
            transaction_sign_and_send(&mut tx, &[&creator, &multisig_create_keypair], rpc_client)
                .await
                .unwrap();

        Ok(result)
    }

    async fn get_ba_multisig(
        multisig: &BaseMultisig,
    ) -> Result<Arc<&dyn BusinessAnalystMultisigTrait>, BaseMultisigError> {
        let multisig: Arc<&dyn BusinessAnalystMultisigTrait> = Arc::new(multisig);

        Ok(multisig)
    }

    pub async fn airdrop(
        rpc_client: &RpcClient,
        address: &Pubkey,
        amount: u64,
    ) -> Result<Signature, Box<dyn Error>> {
        let sig = rpc_client
            .request_airdrop(&address, (amount * LAMPORTS_PER_SOL) as u64)
            .await?;
        println!(
            "🚀Airdropping {} SOL to {} with sig {}",
            amount, address, sig
        );
        loop {
            let confirmed = rpc_client.confirm_transaction(&sig).await?;
            if confirmed {
                break;
            }
        }
        Ok(sig)
    }

    #[tokio::test]
    async fn create_multisig_no_members() -> Result<(), Box<dyn Error>> {
        let rpc_client = RpcClient::new("http://127.0.0.1:8899".to_string());
        let creator: Keypair = Keypair::new();
        let create_key = Keypair::new();

        let _ = airdrop(&rpc_client, &creator.pubkey(), 1).await?;
        let base_multisig = get_base_multisig(&rpc_client, &create_key, &creator, &[])
            .await
            .unwrap();

        assert_eq!(1, base_multisig.get_multisig_members().await.unwrap().len());
        Ok(())
    }

    #[tokio::test]
    async fn add_member() -> Result<(), Box<dyn Error>> {
        let rpc_client = RpcClient::new("http://127.0.0.1:8899".to_string());
        let creator: Keypair = Keypair::new();
        let create_key = Keypair::new();

        let _ = airdrop(&rpc_client, &creator.pubkey(), 1).await?;
        let base_multisig = get_base_multisig(&rpc_client, &create_key, &creator, &[])
            .await
            .unwrap();
        let ba_multisig = get_ba_multisig(&base_multisig).await.unwrap();

        let member = Keypair::new();
        let new_member = Member {
            key: member.pubkey(),
            permissions: Permissions::from_vec(&[Permission::Vote]),
        };
        let mut tx = ba_multisig
            .transaction_add_member(creator.pubkey(), new_member)
            .await
            .unwrap();
        transaction_sign_and_send(&mut tx, &[&creator], &rpc_client)
            .await
            .unwrap();

        let mut tx = ba_multisig
            .transaction_proposal_create(creator.pubkey())
            .await
            .unwrap();
        transaction_sign_and_send(&mut tx, &[&creator], &rpc_client)
            .await
            .unwrap();

        let mut tx = ba_multisig
            .transaction_proposal_approve(creator.pubkey())
            .await
            .unwrap();
        transaction_sign_and_send(&mut tx, &[&creator], &rpc_client)
            .await
            .unwrap();

        let mut tx = ba_multisig
            .transaction_config_transaction_execute(creator.pubkey())
            .await
            .unwrap();
        transaction_sign_and_send(&mut tx, &[&creator], &rpc_client)
            .await
            .unwrap();

        assert_eq!(2, ba_multisig.get_multisig_members().await.unwrap().len());
        Ok(())
    }

    #[tokio::test]
    async fn change_threshold() -> Result<(), Box<dyn Error>> {
        let rpc_client = RpcClient::new("http://127.0.0.1:8899".to_string());
        let creator: Keypair = Keypair::new();
        let create_key = Keypair::new();

        let member = Keypair::new();
        let new_member = Member {
            key: member.pubkey(),
            permissions: Permissions::from_vec(&[Permission::Vote]),
        };

        let _ = airdrop(&rpc_client, &creator.pubkey(), 1).await?;
        let base_multisig = get_base_multisig(&rpc_client, &create_key, &creator, &[new_member])
            .await
            .unwrap();
        let multisig = get_ba_multisig(&base_multisig).await.unwrap();

        let mut tx = multisig
            .transaction_change_threshold(creator.pubkey(), 2)
            .await
            .unwrap();
        transaction_sign_and_send(&mut tx, &[&creator], &rpc_client)
            .await
            .unwrap();

        let mut tx = multisig
            .transaction_proposal_create(creator.pubkey())
            .await
            .unwrap();
        transaction_sign_and_send(&mut tx, &[&creator], &rpc_client)
            .await
            .unwrap();

        let mut tx = multisig
            .transaction_proposal_approve(creator.pubkey())
            .await
            .unwrap();
        transaction_sign_and_send(&mut tx, &[&creator], &rpc_client)
            .await
            .unwrap();

        let mut tx = multisig
            .transaction_config_transaction_execute(creator.pubkey())
            .await
            .unwrap();
        transaction_sign_and_send(&mut tx, &[&creator], &rpc_client)
            .await
            .unwrap();

        assert_eq!(2, multisig.get_threshold().await.unwrap());
        Ok(())
    }

    #[tokio::test]
    async fn vault_transaction_member_approve() -> Result<(), Box<dyn Error>> {
        let rpc_client = RpcClient::new("http://127.0.0.1:8899".to_string());
        let creator: Keypair = Keypair::new();
        let create_key = Keypair::new();

        let member = Keypair::new();
        let new_member = Member {
            key: member.pubkey(),
            permissions: Permissions::from_vec(&[Permission::Vote]),
        };

        let _ = airdrop(&rpc_client, &creator.pubkey(), 1).await?;
        let _ = airdrop(&rpc_client, &member.pubkey(), 1).await?;

        let base_multisig = get_base_multisig(&rpc_client, &create_key, &creator, &[new_member])
            .await
            .unwrap();
        let multisig = get_ba_multisig(&base_multisig).await.unwrap();

        let _ = airdrop(&rpc_client, &multisig.get_vault_pda(), 3).await?;

        let mut tx = multisig
            .transaction_transfer_from_vault(
                creator.pubkey(),
                member.pubkey(),
                2 * LAMPORTS_PER_SOL,
            )
            .await
            .unwrap();
        transaction_sign_and_send(&mut tx, &[&creator], &rpc_client)
            .await
            .unwrap();

        let mut tx = multisig
            .transaction_proposal_create(creator.pubkey())
            .await
            .unwrap();
        transaction_sign_and_send(&mut tx, &[&creator], &rpc_client)
            .await
            .unwrap();

        let mut tx = multisig
            .transaction_proposal_approve(member.pubkey())
            .await
            .unwrap();
        transaction_sign_and_send(&mut tx, &[&member], &rpc_client)
            .await
            .unwrap();

        let mut tx = multisig
            .transaction_vault_transaction_execute(
                creator.pubkey(),
                member.pubkey(),
                2 * LAMPORTS_PER_SOL,
            )
            .await
            .unwrap();
        transaction_sign_and_send(&mut tx, &[&creator], &rpc_client)
            .await
            .unwrap();

        assert!(rpc_client.get_balance(&member.pubkey()).await.unwrap() > 2 * LAMPORTS_PER_SOL);
        assert_eq!(
            rpc_client
                .get_balance(&multisig.get_vault_pda())
                .await
                .unwrap(),
            1 * LAMPORTS_PER_SOL
        );
        Ok(())
    }
}
