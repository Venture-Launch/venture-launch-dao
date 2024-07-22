use crate::dao_module::services::dao_service;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct RemoveMemberDaoSchema {
    multisig_pda: String,
    pubkey: String
}

pub async fn consume(request: RemoveMemberDaoSchema) -> Result<String, String> {
    let pda = dao_service::remove_member(request.multisig_pda.clone(), request.pubkey).await.unwrap();
    return Ok(format!(
        "\"multisig_pda\": \"{}\",
    {pda}", request.multisig_pda
    ));
}
