use async_trait::async_trait;
use super::{base_multisig::{BaseMultisig, BaseMultisigCreateArgs}, base_multisig_trait::BaseMultisigTrait, error::BaseMultisigError};

#[async_trait]
pub trait InvestorMultisigTrait<Args = BaseMultisigCreateArgs> : BaseMultisigTrait<Args, Error = BaseMultisigError>{

}

#[async_trait]
impl InvestorMultisigTrait<BaseMultisigCreateArgs> for BaseMultisig {

}


#[cfg(test)]
mod tests {
    use std::{error::Error, sync::Arc};

    use crate::multisig_utils::{business_analyst_multisig_trait::BusinessAnalystMultisigTrait, error::BaseMultisigError};

    use super::*;
    use solana_client::nonblocking::rpc_client::RpcClient;
    use solana_sdk::{native_token::LAMPORTS_PER_SOL, pubkey::Pubkey, signature::{Keypair, Signature}, signer::Signer, transaction::Transaction};
    use squads_multisig::{squads_multisig_program, state::ProposalStatus};
    use squads_multisig_program::{Member, Permission, Permissions};
    use tokio;

    async fn transaction_sign_and_send(tx: &mut Transaction, keys: &[&Keypair], multisig_rpc: &RpcClient) -> Result<(), Box<dyn Error>> {
        let recent_blockhash = multisig_rpc.get_latest_blockhash().await.unwrap();
        let _ = tx.try_sign(keys, recent_blockhash);
        let _ = multisig_rpc.send_and_confirm_transaction(tx).await?;
        Ok(())
    }

    async fn get_base_multisig(rpc_client: &RpcClient, multisig_create_keypair: &Keypair, creator: &Keypair, members: &[Member]) -> Result<BaseMultisig, BaseMultisigError> {
        let result = BaseMultisig::new(BaseMultisigCreateArgs {
            rpc_client: RpcClient::new(rpc_client.url()),
            multisig_create_keypair: multisig_create_keypair.insecure_clone(),
            creator: creator.pubkey().clone()
        }).await?;

        let mut tx = result.transaction_create_multisig(members, 1, 0, multisig_create_keypair).await?;
        let _ = transaction_sign_and_send(&mut tx, &[&creator, &multisig_create_keypair], rpc_client).await.unwrap();

        Ok(result)
    }

    async fn get_ba_multisig(multisig: &BaseMultisig) -> Result<Arc<&dyn BusinessAnalystMultisigTrait>, BaseMultisigError> {
        let multisig : Arc<&dyn BusinessAnalystMultisigTrait> = Arc::new(multisig);

        Ok(multisig)
    }

    async fn get_investor_multisig(multisig: &BaseMultisig) -> Result<Arc<&dyn InvestorMultisigTrait>, BaseMultisigError> {
        let multisig : Arc<&dyn InvestorMultisigTrait> = Arc::new(multisig);

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
        let base_multisig = get_base_multisig(&rpc_client, &create_key, &ba, &[investor]).await.unwrap();
        let investor_multisig = get_investor_multisig(&base_multisig).await.unwrap();

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
        let base_multisig = get_base_multisig(&rpc_client, &create_key, &ba, &[investor]).await.unwrap();
        let ba_multisig = get_ba_multisig(&base_multisig).await.unwrap();
        let investor_multisig = get_investor_multisig(&base_multisig).await.unwrap();

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

        let base_multisig = get_base_multisig(&rpc_client, &create_key, &ba, &[investor]).await.unwrap();
        let ba_multisig = get_ba_multisig(&base_multisig).await.unwrap();
        let investor_multisig = get_investor_multisig(&base_multisig).await.unwrap();

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