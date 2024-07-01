use crate::dao_module::services::dao_service;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct CreateUserSchema {
    email: String,
    password: String,
    employee_id: i32,
}

pub async fn consume(request: CreateUserSchema) -> Result<String, String> {
    dao_service::create_dao();
    return Ok(format!(
        "User {} created successfully with roles:",
        request.email
    ));
}
