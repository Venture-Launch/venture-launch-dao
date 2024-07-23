use crate::dao_module::services::dao_service;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct VoteDaoSchema {
    multisig_pda: String,
    voter: String,
    vote: String
}

pub async fn consume(request: VoteDaoSchema) -> Result<String, String> {
    let pda = dao_service::vote(request.multisig_pda.clone(), request.voter, request.vote).await.unwrap();
    return Ok(format!(
        "\"multisig_pda\": \"{}\",
        {pda}", request.multisig_pda
    ));
}
