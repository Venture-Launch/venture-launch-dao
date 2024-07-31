use crate::dao_module::services::dao_service;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ProposalDaoSchema {
    project_id: String,
    proposal: String
}

pub async fn consume(request: ProposalDaoSchema) -> Result<String, String> {
    let pda = dao_service::create_dao().await.unwrap();
    return Ok(format!(
        "Dao {}: {} created successfully",
        request.project_id,
        pda
    ));
}
