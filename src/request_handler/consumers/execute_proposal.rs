use crate::dao_module::services::dao_service;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ProposalExecuteDaoSchema {
    project_id: String
}

pub async fn consume(request: ProposalExecuteDaoSchema) -> Result<String, String> {
    let pda = dao_service::execute_proposal(request.project_id.clone()).await.unwrap();
    return Ok(format!(
        "Dao {}: {} created successfully",
        request.project_id,
        pda
    ));
}
