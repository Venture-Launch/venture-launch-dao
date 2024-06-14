pub mod multisig;
pub mod error;

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;
    use {error::MultisigError, multisig::{InvestorMultisigCreateArgs, InvestorsMultisig}};
    use squads_multisig::{solana_client::nonblocking::rpc_client::RpcClient, state::{Member, Permission, Permissions}};
    use solana_sdk::{self, message::Message, native_token::LAMPORTS_PER_SOL, pubkey::Pubkey, signature::{Keypair, Signature}, signer::Signer, transaction::Transaction};
    use std::error::Error;

    pub async fn get_multisig_no_members(rpc_client: RpcClient, multisig_create_keypair: Keypair, creator: Keypair) -> Result<InvestorsMultisig, MultisigError> {
        let create_key = multisig_create_keypair;

        let _ = airdrop(&rpc_client, &creator.pubkey(), 2).await;

        let multisig = InvestorsMultisig::new(InvestorMultisigCreateArgs {
            rpc_client: RpcClient::new(String::from("http://127.0.0.1:8899")),
            multisig_create_keypair: create_key.insecure_clone(),
            creator: creator.pubkey()
        }).await?;

        let ix = multisig.instruction_create_multisig(&[], 1, 0);

        let mut message = Message::new(&[ix], Some(&creator.pubkey()));    //Creator.pubkey()));
        let recent_blockhash = rpc_client.get_latest_blockhash().await.unwrap();
        message.recent_blockhash = recent_blockhash;

        let mut transaction = Transaction::new_unsigned(message);
        let _ = transaction.try_sign(&[&creator, &create_key], recent_blockhash);

        let _ = rpc_client.send_and_confirm_transaction(&transaction).await.unwrap();

        Ok(multisig)
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
    async fn multisig_new() -> Result<(), MultisigError> {
        let rpc_client = RpcClient::new(String::from("http://127.0.0.1:8899"));

        let create_key = Keypair::new();
        let creator = Keypair::new();

        InvestorsMultisig::new(InvestorMultisigCreateArgs {
            rpc_client: rpc_client,
            multisig_create_keypair: create_key,
            creator: creator.pubkey()
        }).await?;

        Ok(())
    }

    #[tokio::test]
    async fn create_multisig_v2_via_instruction() -> Result<(), Box<dyn Error>> {
        let rpc_client = RpcClient::new(String::from("http://127.0.0.1:8899"));

        let create_key = Keypair::new();
        let creator = Keypair::new();
        let member = Keypair::new();

        let _ = airdrop(&rpc_client, &creator.pubkey(), 2).await;

        let multisig = InvestorsMultisig::new(InvestorMultisigCreateArgs {
            rpc_client: RpcClient::new(String::from("http://127.0.0.1:8899")),
            multisig_create_keypair: create_key.insecure_clone(),
            creator: creator.pubkey()
        }).await?;

        let ix = multisig.instruction_create_multisig(&[], 1, 0);

        let mut message = Message::new(&[ix], Some(&creator.pubkey()));    //Creator.pubkey()));
        let recent_blockhash = rpc_client.get_latest_blockhash().await.unwrap();
        message.recent_blockhash = recent_blockhash;

        let mut transaction = Transaction::new_unsigned(message);
        let _ = transaction.try_sign(&[&creator, &create_key], recent_blockhash);

        let _ = rpc_client.send_and_confirm_transaction(&transaction).await?;

        Ok(())
    }

    #[tokio::test]
    async fn create_multisig_v2_via_transaction() -> Result<(), Box<dyn Error>> {
        let rpc_client = RpcClient::new(String::from("http://127.0.0.1:8899"));

        let create_key = Keypair::new();
        let creator = Keypair::new();
        let member = Keypair::new();

        let _ = airdrop(&rpc_client, &creator.pubkey(), 2).await;

        let multisig = InvestorsMultisig::new(InvestorMultisigCreateArgs {
            rpc_client: RpcClient::new(String::from("http://127.0.0.1:8899")),
            multisig_create_keypair: create_key.insecure_clone(),
            creator: creator.pubkey()
        }).await?;

        let mut tx: Transaction = multisig.transaction_create_multisig(&[], 1, 0).await.unwrap();

        let recent_blockhash = rpc_client.get_latest_blockhash().await.unwrap();
        let _ = tx.try_sign(&[&creator, &create_key], recent_blockhash);

        let _ = rpc_client.send_and_confirm_transaction(&tx).await?;

        Ok(())
    }

    #[tokio::test]
    async fn create_multisig_with_member() -> Result<(), Box<dyn Error>> {
        let rpc_client = RpcClient::new(String::from("http://127.0.0.1:8899"));

        let create_key = Keypair::new();
        let creator = Keypair::new();
        let member = Keypair::new();

        let _ = airdrop(&rpc_client, &creator.pubkey(), 2).await;

        let multisig = InvestorsMultisig::new(InvestorMultisigCreateArgs {
            rpc_client: RpcClient::new(String::from("http://127.0.0.1:8899")),
            multisig_create_keypair: create_key.insecure_clone(),
            creator: creator.pubkey()
        }).await?;

        let mut tx: Transaction = multisig.transaction_create_multisig(&[
            squads_multisig::state::Member {
                key: member.pubkey(),
                permissions: Permissions::from_vec(&[Permission::Initiate, Permission::Vote, Permission::Execute]),
            }
        ], 1, 0).await.unwrap();

        let recent_blockhash = rpc_client.get_latest_blockhash().await.unwrap();
        let _ = tx.try_sign(&[&creator, &create_key], recent_blockhash);

        let _ = rpc_client.send_and_confirm_transaction(&tx).await?;

        let members = multisig.get_multisig_members().await?;

        assert_eq!(2, members.len());
        Ok(())
    }

    #[tokio::test]
    async fn add_member_to_multisig() -> Result<(), Box<dyn Error>> {
        let rpc_client = RpcClient::new(String::from("http://127.0.0.1:8899"));

        let create_key = Keypair::new();
        let creator = Keypair::new();
        let new_member = Keypair::new();

        let new_member = Member {
            key: new_member.pubkey(),
            permissions: Permissions::from_vec(&[Permission::Vote, Permission::Initiate])
        };

        let multisig = get_multisig_no_members(rpc_client, create_key.insecure_clone(), creator.insecure_clone()).await.unwrap();

        let mut tx = multisig.transaction_add_member_config_transaction(creator.pubkey(), new_member).await.unwrap();
        let recent_blockhash = multisig.create_args.rpc_client.get_latest_blockhash().await.unwrap();
        let _ = tx.try_sign(&[&creator], recent_blockhash);
        let _ = multisig.create_args.rpc_client.send_and_confirm_transaction(&tx).await?;

        let mut tx = multisig.transaction_proposal_create(creator.pubkey()).await.unwrap();
        let recent_blockhash = multisig.create_args.rpc_client.get_latest_blockhash().await.unwrap();
        let _ = tx.try_sign(&[&creator], recent_blockhash);
        let _ = multisig.create_args.rpc_client.send_and_confirm_transaction(&tx).await?;

        let mut tx = multisig.transaction_proposal_approve(creator.pubkey()).await.unwrap();
        let recent_blockhash = multisig.create_args.rpc_client.get_latest_blockhash().await.unwrap();
        let _ = tx.try_sign(&[&creator], recent_blockhash);
        let _ = multisig.create_args.rpc_client.send_and_confirm_transaction(&tx).await?;

        let members: Vec<Member> = multisig.get_multisig_members().await?;
        assert_eq!(1, members.len());

        let mut tx = multisig.transaction_config_transaction_execute(creator.pubkey()).await.unwrap();
        let recent_blockhash = multisig.create_args.rpc_client.get_latest_blockhash().await.unwrap();
        let _ = tx.try_sign(&[&creator], recent_blockhash);
        let _ = multisig.create_args.rpc_client.send_and_confirm_transaction(&tx).await?;


        let members: Vec<Member> = multisig.get_multisig_members().await?;
        assert_eq!(2, members.len());

        Ok(())
    }
}