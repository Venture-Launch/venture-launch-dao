use crate::dao_module::services::dao_service;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct AddMemberDaoSchema {
    project_id: String,
    pubkey: String,
    permissions: Vec<String>
}

pub async fn consume(request: AddMemberDaoSchema) -> Result<String, String> {
    let pda = dao_service::add_member(request.project_id.clone(), request.pubkey, request.permissions).await.unwrap();
    return Ok(format!(
        "Dao {}: {} created successfully",
        request.project_id,
        pda
    ));
}
