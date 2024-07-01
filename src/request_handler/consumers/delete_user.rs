use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct DeleteUserSchema {
    employee_id: i32,
}

pub async fn consume(request: DeleteUserSchema) -> Result<String, String> {
    Ok(format!(
        "User with employee_id={} deleted successfully",
        request.employee_id
    ))
}
