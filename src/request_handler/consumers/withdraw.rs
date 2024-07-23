use crate::dao_module::services::dao_service;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct WithdrawDaoSchema {
    multisig_pda: String,
    proposal_id: String,
    is_execute: bool,
    receiver: String,
    amount: u64
}


pub async fn consume(request: WithdrawDaoSchema) -> Result<String, String> {
    let pda = dao_service::withdraw(request.multisig_pda.clone(), request.is_execute, request.receiver, request.amount).await.unwrap();
    return Ok(format!(
        "\"multisig_pda\": \"{}\",
        \"proposal_id\": \"{}\",
        {pda}", request.multisig_pda, request.proposal_id
    ));
}
