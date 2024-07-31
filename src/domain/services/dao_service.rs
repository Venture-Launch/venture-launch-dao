use std::error::Error;
use std::str::FromStr;
use std::sync::Arc;

use dotenv::dotenv;

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signature};
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;
use spl_associated_token_account::get_associated_token_address;
use spl_associated_token_account::instruction::create_associated_token_account;
use squads_multisig::state::{Member, Permission, Permissions};


use crate::domain::entities::multisig::base_multisig::{BaseMultisig, BaseMultisigCreateArgs, BaseMultisigInitArgs};
use crate::domain::entities::multisig::base_multisig_trait::BaseMultisigTrait;
use crate::domain::entities::multisig::business_analyst_multisig_trait::BusinessAnalystMultisigTrait;
use crate::domain::entities::multisig::investor_multisig_trait::InvestorMultisigTrait;

async fn get_ba_keypair() -> Result<Keypair, String> {
    dotenv().ok();

    let private_key_str = std::env::var("BA_PRIVATE_KEY").expect("BA_PRIVATE_KEY not found");
    let private_key_vec: Vec<u8> = private_key_str.split(',')
                                                            .map(|s| s.parse().expect("Invalid number"))
                                                            .collect();

    let creator_keypair = Keypair::from_bytes(&private_key_vec).expect("Invalid SecretKey");

    Ok(creator_keypair)
}
fn get_rpc_client() -> Result<RpcClient, String> {
    dotenv().ok();

    Ok(RpcClient::new(std::env::var("DEFAULT_RPC_CLIENT").unwrap_or_else(|_| "http://127.0.0.1:8899".into()).to_string()))
}
async fn create_base_multisig(create_key: &Keypair) -> Result<BaseMultisig, String> {

    let rpc_client: RpcClient = get_rpc_client().map_err(|err| format!("\"msg\": \"{err}\""))?;
    let creator_keypair: Keypair = get_ba_keypair().await.map_err(|err| format!("\"msg\": \"{err}\""))?;

    println!("creator: {}", creator_keypair.pubkey());
    println!("balance: {}", rpc_client.get_balance(&creator_keypair.pubkey()).await.unwrap());
    let multisig = BaseMultisig::new(BaseMultisigCreateArgs{
        rpc_client,
        multisig_create_keypair: create_key.insecure_clone(),
        creator: creator_keypair.pubkey()
    }).await.map_err(|err| format!("\"msg\": \"{err}\""))?;

    let token_program_id = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap();
    let token_mint = Pubkey::from_str("Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr").unwrap();

    println!("{token_program_id}");
    println!("{token_mint}");

    let ix = create_associated_token_account(
        &creator_keypair.pubkey(),
        &multisig.get_vault_pda(),
        &token_mint,
        &token_program_id
    );

    let mut transaction = Transaction::new_with_payer(
        &[ix],
        Some(&creator_keypair.pubkey()),
    );

    let recent_blockhash = multisig.get_rpc_client().get_latest_blockhash().await.unwrap();
    let _ = transaction.try_sign(&[&creator_keypair], recent_blockhash);
    let _ = multisig.get_rpc_client().send_and_confirm_transaction(&transaction).await.expect("Failed to send transaction");

    let associated_token_address = get_associated_token_address(&multisig.get_vault_pda(), &token_mint);
    let account_info = multisig.get_rpc_client().get_account(&associated_token_address).await;
    if let Ok(_) = account_info {
        println!("Associated token account already exists: {}", associated_token_address);
    }

    Ok(multisig)
}

async fn get_base_multisig(multisig_pda: Pubkey) -> Result<BaseMultisig, String> {
    dotenv().ok();

    let rpc_client: RpcClient = get_rpc_client().map_err(|err| format!("\"msg\": \"{err}\""))?;

    let creator_keypair = get_ba_keypair().await.map_err(|err| format!("\"msg\": \"{err}\""))?;

    let multisig = BaseMultisig::from_multisig_pda(BaseMultisigInitArgs {
        rpc_client,
        multisig_pda,
        creator: creator_keypair.pubkey()
    }).await.map_err(|err| format!("\"msg\": \"{err}\""))?;

    Ok(multisig)
}


/// debug code, works only on localhost
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

pub async fn create_dao() -> Result<String, String> {
    let create_key = Keypair::new();
    let creator_keypair = get_ba_keypair().await.map_err(|err| format!("\"msg\": \"{err}\""))?;

    let multisig = create_base_multisig(&create_key).await.map_err(|err| format!("\"msg\": \"{err}\""))?;

    let multisig: Arc<&dyn BusinessAnalystMultisigTrait> = Arc::new(&multisig);

    let mut tx = multisig.transaction_create_multisig(&[], 1, 0, &create_key).await.map_err(|err| format!("\"msg\": \"{err}\""))?;
    let recent_blockhash = multisig.get_rpc_client().get_latest_blockhash().await.map_err(|err| format!("\"msg\": \"{err}\""))?;
    println!("before try_sign");
    let _ = tx.try_sign(&[&creator_keypair, &create_key], recent_blockhash);
    let _ = multisig.get_rpc_client().send_and_confirm_transaction(&tx).await.map_err(|err| format!("\"msg\": \"{err}\""))?;
    println!("after confirm");
    println!("multisig: {}", multisig.get_multisig_pda());

    Ok(format!(
        "\"multisig_pda\": \"{}\",
        \"vault_pda\":  \"{}\",
        \"threshold\":  \"{}\"",
        multisig.get_multisig_pda().to_string(), multisig.get_vault_pda().to_string(), multisig.get_threshold().await.unwrap()
    ))
}

pub async fn add_member(
    multisig_pda: String,
    pubkey: String,
    _permissions: Vec<String>
) -> Result<String, String>  {
    dotenv().ok();

    let creator_keypair = get_ba_keypair().await?;
    let multisig_pda = Pubkey::from_str(&multisig_pda).map_err(|err| format!("\"msg\": \"{err}\""))?;
    let multisig = get_base_multisig(multisig_pda).await.map_err(|err| format!("\"msg\": \"{err}\""))?;

    let multisig: Arc<&dyn BusinessAnalystMultisigTrait> = Arc::new(&multisig);

    let new_member_pubkey = Pubkey::from_str(pubkey.as_str()).map_err(|err| format!("\"msg\": \"{err}\""))?;
    let new_member = Member {
        key: new_member_pubkey,
        permissions: Permissions::from_vec(&[Permission::Vote]),
    };

    let mut tx = multisig.transaction_add_member(creator_keypair.pubkey(), new_member).await.map_err(|err| format!("\"msg\": \"{err}\""))?;
    // let mut tx = multisig.get_transaction_from_instructions(creator_keypair.pubkey(), &[ix_add_member]).await.map_err(|err| format!("\"msg\": \"{err}\""))?;
    let recent_blockhash = multisig.get_rpc_client().get_latest_blockhash().await.map_err(|err| format!("\"msg\": \"{err}\""))?;
    let _ = tx.try_sign(&[&creator_keypair], recent_blockhash);
    let sig = multisig.get_rpc_client().send_and_confirm_transaction(&tx).await.map_err(|err| format!("\"msg\": \"{err}\""))?;
    println!("sig: {}", sig);

    let ix_prpose = multisig.instruction_proposal_create(creator_keypair.pubkey()).await.map_err(|err| format!("\"msg\": \"{err}\""))?;
    let ix_approve = multisig.instruction_proposal_approve(creator_keypair.pubkey()).await.map_err(|err| format!("\"msg\": \"{err}\""))?;
    let ix_exec = multisig.instruction_config_transaction_execute(creator_keypair.pubkey()).await.map_err(|err| format!("\"msg\": \"{err}\""))?;
    let mut tx = multisig.get_transaction_from_instructions(creator_keypair.pubkey(), &[ix_prpose, ix_approve, ix_exec]).await.map_err(|err| format!("\"msg\": \"{err}\""))?;
    let recent_blockhash = multisig.get_rpc_client().get_latest_blockhash().await.map_err(|err| format!("\"msg\": \"{err}\""))?;
    let _ = tx.try_sign(&[&creator_keypair], recent_blockhash);
    let sig = multisig.get_rpc_client().send_and_confirm_transaction(&tx).await.map_err(|err| format!("\"msg\": \"{err}\""))?;
    println!("sig: {}", sig);

    // let mems = multisig.get_multisig_members().await.map_err(|err| format!("\"msg\": \"{err}\""))?;

    // for mem in mems {
    //     println!("{}", mem.key);
    // }

    Ok(
        format!(
            "\"member\":  \"{}\"",
            new_member_pubkey
        )
    )
}

pub async fn remove_member(
    multisig_pda: String,
    pubkey: String
) -> Result<String, String>  {
    dotenv().ok();

    let creator_keypair = get_ba_keypair().await.map_err(|err| format!("\"msg\": \"{err}\""))?;
    let multisig_pda = Pubkey::from_str(&multisig_pda).map_err(|err| format!("\"msg\": \"{err}\""))?;
    let multisig = get_base_multisig(multisig_pda).await.map_err(|err| format!("\"msg\": \"{err}\""))?;

    let multisig: Arc<&dyn BusinessAnalystMultisigTrait> = Arc::new(&multisig);
    let old_member_pubkey = Pubkey::from_str(pubkey.as_str()).map_err(|err| format!("\"msg\": \"{err}\""))?;

    let ix_remove_member = multisig.instructions_remove_member(creator_keypair.pubkey(), old_member_pubkey).await.map_err(|err| format!("\"msg\": \"{err}\""))?;
    let mut tx = multisig.get_transaction_from_instructions(creator_keypair.pubkey(), &[ix_remove_member]).await.map_err(|err| format!("\"msg\": \"{err}\""))?;
    let recent_blockhash = multisig.get_rpc_client().get_latest_blockhash().await.map_err(|err| format!("\"msg\": \"{err}\""))?;
    let _ = tx.try_sign(&[&creator_keypair], recent_blockhash);
    let _ = multisig.get_rpc_client().send_and_confirm_transaction(&tx).await.map_err(|err| format!("\"msg\": \"{err}\""))?;

    let ix_prpose = multisig.instruction_proposal_create(creator_keypair.pubkey()).await.map_err(|err| format!("\"msg\": \"{err}\""))?;
    let ix_approve = multisig.instruction_proposal_approve(creator_keypair.pubkey()).await.map_err(|err| format!("\"msg\": \"{err}\""))?;
    let ix_exec = multisig.instruction_config_transaction_execute(creator_keypair.pubkey()).await.map_err(|err| format!("\"msg\": \"{err}\""))?;
    let mut tx = multisig.get_transaction_from_instructions(creator_keypair.pubkey(), &[ix_prpose, ix_approve, ix_exec]).await.map_err(|err| format!("\"msg\": \"{err}\""))?;
    let recent_blockhash = multisig.get_rpc_client().get_latest_blockhash().await.map_err(|err| format!("\"msg\": \"{err}\""))?;
    let _ = tx.try_sign(&[&creator_keypair], recent_blockhash);
    let sig = multisig.get_rpc_client().send_and_confirm_transaction(&tx).await.map_err(|err| format!("\"msg\": \"{err}\""))?;
    println!("sig: {}", sig);

    Ok(
        format!(
            "\"member\":  \"{}\"",
            old_member_pubkey
        )
    )
}

pub async fn change_threshold(
    multisig_pda: String,
    new_threshold: u16
) -> Result<String, String>  {
    dotenv().ok();

    let creator_keypair = get_ba_keypair().await.map_err(|err| format!("\"msg\": \"{err}\""))?;
    let multisig_pda = Pubkey::from_str(&multisig_pda).map_err(|err| format!("\"msg\": \"{err}\""))?;
    let multisig = get_base_multisig(multisig_pda).await.map_err(|err| format!("\"msg\": \"{err}\""))?;

    let multisig: Arc<&dyn BusinessAnalystMultisigTrait> = Arc::new(&multisig);

    let ix_change_threshold = multisig.instruction_change_threshold(creator_keypair.pubkey(), new_threshold).await.map_err(|err| format!("\"msg\": \"{err}\""))?;
    let mut tx = multisig.get_transaction_from_instructions(creator_keypair.pubkey(), &[ix_change_threshold]).await.map_err(|err| format!("\"msg\": \"{err}\""))?;
    let recent_blockhash = multisig.get_rpc_client().get_latest_blockhash().await.map_err(|err| format!("\"msg\": \"{err}\""))?;
    let _ = tx.try_sign(&[&creator_keypair], recent_blockhash);
    let _ = multisig.get_rpc_client().send_and_confirm_transaction(&tx).await.map_err(|err| format!("\"msg\": \"{err}\""))?;

    let ix_prpose = multisig.instruction_proposal_create(creator_keypair.pubkey()).await.map_err(|err| format!("\"msg\": \"{err}\""))?;
    let ix_approve = multisig.instruction_proposal_approve(creator_keypair.pubkey()).await.map_err(|err| format!("\"msg\": \"{err}\""))?;
    let ix_exec = multisig.instruction_config_transaction_execute(creator_keypair.pubkey()).await.map_err(|err| format!("\"msg\": \"{err}\""))?;
    let mut tx = multisig.get_transaction_from_instructions(creator_keypair.pubkey(), &[ix_prpose, ix_approve, ix_exec]).await.map_err(|err| format!("\"msg\": \"{err}\""))?;
    let recent_blockhash = multisig.get_rpc_client().get_latest_blockhash().await.map_err(|err| format!("\"msg\": \"{err}\""))?;
    let _ = tx.try_sign(&[&creator_keypair], recent_blockhash);
    let sig = multisig.get_rpc_client().send_and_confirm_transaction(&tx).await.map_err(|err| format!("\"msg\": \"{err}\""))?;
    println!("sig: {}", sig);

    Ok(
        format!(
            "\"new_threshold\":  \"{}\"",
            new_threshold
        )
    )
}

pub async fn execute_proposal(
    multisig_pda: String
) -> Result<String, String>  {
    dotenv().ok();

    let creator_keypair = get_ba_keypair().await.map_err(|err| format!("\"msg\": \"{err}\""))?;
    let multisig_pda = Pubkey::from_str(&multisig_pda).map_err(|err| format!("\"msg\": \"{err}\""))?;
    let multisig = get_base_multisig(multisig_pda).await.map_err(|err| format!("\"msg\": \"{err}\""))?;

    let multisig: Arc<&dyn BusinessAnalystMultisigTrait> = Arc::new(&multisig);

    let mut tx = multisig.transaction_config_transaction_execute(creator_keypair.pubkey()).await.map_err(|err| format!("\"msg\": \"{err}\""))?;
    let recent_blockhash = multisig.get_rpc_client().get_latest_blockhash().await.map_err(|err| format!("\"msg\": \"{err}\""))?;
    let _ = tx.try_sign(&[&creator_keypair], recent_blockhash);
    let _ = multisig.get_rpc_client().send_and_confirm_transaction(&tx).await.map_err(|err| format!("\"msg\": \"{err}\""))?;

    Ok(
        format!(
            "\"propose\":  \"success\""
        )
    )
}

pub async fn vote(
    multisig_pda: String,
    voter: String,
    vote: String
) -> Result<String, String>  {
    dotenv().ok();

    let creator_keypair = get_ba_keypair().await.map_err(|err| format!("\"msg\": \"{err}\""))?;
    let multisig_pda = Pubkey::from_str(&multisig_pda).map_err(|err| format!("\"msg\": \"{err}\""))?;
    let multisig = get_base_multisig(multisig_pda).await.map_err(|err| format!("\"msg\": \"{err}\""))?;

    let voter = Pubkey::from_str(voter.as_str()).map_err(|err| format!("\"msg\": \"{err}\""))?;

    let multisig: Arc<&dyn InvestorMultisigTrait> = Arc::new(&multisig);

    let mut tx = match vote.as_str() {
        "Cancel" => {
            multisig.transaction_proposal_cancel(creator_keypair.pubkey()).await.unwrap()
        },
        "Approve" => {
            multisig.transaction_proposal_approve(creator_keypair.pubkey()).await.unwrap()
        },
        vote => {
            return Err(format!("{vote} is not an \"Approve\" or \"Cancel\""));
        }
    };

    let recent_blockhash = multisig.get_rpc_client().get_latest_blockhash().await.map_err(|err| format!("\"msg\": \"{err}\""))?;
    let _ = tx.try_sign(&[&creator_keypair], recent_blockhash);
    let _ = multisig.get_rpc_client().send_and_confirm_transaction(&tx).await.map_err(|err| format!("\"msg\": \"{err}\""))?;

    Ok(
        format!(
            "\"voter\":  \"{}\",
            \"vote\":  \"{}\"",
            voter,
            vote
        )
    )
}

pub async fn withdraw(
    multisig_pda: String,
    is_execute: bool,
    receiver: String,
    amount: u64
) -> Result<String, String>  {
    dotenv().ok();

    let creator_keypair = get_ba_keypair().await.unwrap();
    let multisig_pda = Pubkey::from_str(&multisig_pda).unwrap();
    let multisig = get_base_multisig(multisig_pda).await.unwrap();

    let multisig: Arc<&dyn BusinessAnalystMultisigTrait> = Arc::new(&multisig);

    let receiver = Pubkey::from_str(&receiver).unwrap();

    if is_execute == true {
        let mut tx = multisig.transaction_vault_transaction_execute(creator_keypair.pubkey(), receiver, amount).await.unwrap();
        let recent_blockhash = multisig.get_rpc_client().get_latest_blockhash().await.unwrap();
        let _ = tx.try_sign(&[&creator_keypair], recent_blockhash);
        let _ = multisig.get_rpc_client().send_and_confirm_transaction(&tx).await.unwrap();

        return Ok(
            format!(
                "\"is_execute\":  \"{}\",
                \"receiver\":  \"{}\",
                \"amount\":  \"{}\"",
                is_execute,
                receiver,
                amount
            )
        )
    }

    // let finance = multisig.get_rpc_client().get_balance(&multisig.get_vault_pda()).await.unwrap();;
    // println!("vault: {}", finance);

    let mut tx = multisig.transaction_transfer_from_vault(creator_keypair.pubkey(), receiver, amount).await.unwrap();
    let recent_blockhash = multisig.get_rpc_client().get_latest_blockhash().await.unwrap();
    let _ = tx.try_sign(&[&creator_keypair], recent_blockhash);
    let _ = multisig.get_rpc_client().send_and_confirm_transaction(&tx).await.unwrap();

    let ix_propose = multisig.instruction_proposal_create(creator_keypair.pubkey()).await.unwrap();
    let ix_approve = multisig.instruction_proposal_approve(creator_keypair.pubkey()).await.unwrap();
    let mut tx = multisig.get_transaction_from_instructions(creator_keypair.pubkey(), &[ix_propose, ix_approve]).await.unwrap();
    let recent_blockhash = multisig.get_rpc_client().get_latest_blockhash().await.unwrap();
    let _ = tx.try_sign(&[&creator_keypair], recent_blockhash);
    let _ = multisig.get_rpc_client().send_and_confirm_transaction(&tx).await.unwrap();

    Ok(
        format!(
            "\"is_execute\":  \"{}\",
            \"receiver\":  \"{}\",
            \"amount\":  \"{}\"",
            is_execute,
            receiver,
            amount
        )
    )
}