use crate::dao_module::services::dao_service;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ChangeThresholdDaoSchema {
    multisig_pda: String,
    new_threshold: u16
}

pub async fn consume(request: ChangeThresholdDaoSchema) -> Result<String, String> {
    let pda = dao_service::change_threshold(request.multisig_pda.clone(), request.new_threshold).await.unwrap();
    return Ok(format!(
        "\"multisig_pda\": \"{}\",
        {pda}", request.multisig_pda
    ));
}
