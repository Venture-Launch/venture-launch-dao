use crate::dao_module::services::dao_service;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct VoteDaoSchema {
    project_id: String,
    voter: String,
    vote: String
}

pub async fn consume(request: VoteDaoSchema) -> Result<String, String> {
    let pda = dao_service::vote(request.project_id.clone(), request.voter, request.vote).await.unwrap();
    return Ok(format!(
        "Dao {}: {} created successfully",
        request.project_id,
        pda
    ));
}
