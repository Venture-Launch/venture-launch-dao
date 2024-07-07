use crate::dao_module::services::dao_service;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct WithdrawDaoSchema {
    project_id: String,
    is_execute: bool,
    receiver: String,
    amount: u64
}


pub async fn consume(request: WithdrawDaoSchema) -> Result<String, String> {
    let pda = dao_service::withdraw(request.project_id.clone(), request.is_execute, request.receiver, request.amount).await.unwrap();
    return Ok(format!(
        "Dao {}: {} created successfully",
        request.project_id,
        pda
    ));
}
