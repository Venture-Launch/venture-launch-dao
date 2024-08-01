use crate::domain::services::dao_service;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct AddMemberDaoSchema {
    multisig_pda: String,
    pubkey: String,
    permissions: Vec<String>
}

pub async fn consume(request: AddMemberDaoSchema) -> Result<String, String> {
    let pda = dao_service::add_member(request.multisig_pda.clone(), request.pubkey, request.permissions).await.unwrap();
    return Ok(format!(
        "\"multisig_pda\": \"{}\",
    {pda}", request.multisig_pda
    ));
}