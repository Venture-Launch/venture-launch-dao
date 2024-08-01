use crate::domain::services::dao_service;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct CreateDaoSchema {
    project_id: String,
}

pub async fn consume(request: CreateDaoSchema) -> Result<String, String> {
    let pda = dao_service::create_dao().await.unwrap();

    return Ok(format!(
        "{pda},
        \"project_id\":  \"{}\"", request.project_id
    ));
}
