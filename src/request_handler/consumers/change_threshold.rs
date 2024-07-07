use crate::dao_module::services::dao_service;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ChangeThresholdDaoSchema {
    project_id: String,
    new_threshold: u16
}

pub async fn consume(request: ChangeThresholdDaoSchema) -> Result<String, String> {
    let pda = dao_service::change_threshold(request.project_id.clone(), request.new_threshold).await.unwrap();
    return Ok(format!(
        "Dao {}: {} created successfully",
        request.project_id,
        pda
    ));
}
