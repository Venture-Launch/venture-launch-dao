use crate::dao_module::services::dao_service;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct RemoveMemberDaoSchema {
    project_id: String,
    pubkey: String
}

pub async fn consume(request: RemoveMemberDaoSchema) -> Result<String, String> {
    let pda = dao_service::remove_member(request.project_id.clone(), request.pubkey).await.unwrap();
    return Ok(format!(
        "Dao {}: {} created successfully",
        request.project_id,
        pda
    ));
}
