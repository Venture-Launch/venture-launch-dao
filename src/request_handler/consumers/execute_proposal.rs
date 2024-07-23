use crate::dao_module::services::dao_service;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ProposalExecuteDaoSchema {
    multisig_pda: String
}

pub async fn consume(request: ProposalExecuteDaoSchema) -> Result<String, String> {
    let pda = dao_service::execute_proposal(request.multisig_pda.clone()).await.unwrap();
    return Ok(format!(
        "\"multisig_pda\": \"{}\",
        {pda}", request.multisig_pda
    ));
}
