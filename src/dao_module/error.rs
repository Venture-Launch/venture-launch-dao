use thiserror::Error;
use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum InvestorMultisigError {
    #[error("Failed to fetch program config account")]
    FailedToFetchProgramConfigAccount,
    #[error("Failed to fetch multisig config account")]
    FailedToFetchMultisigConfigAccount,
    #[error("Failed to fetch proposal config account")]
    FailedToFetchProposalConfigAccount,
    #[error("Failed to deserialize multisig config data")]
    FailedToDeserializeMultisigConfigData,
    #[error("Failed to deserialize program config account")]
    FailedToDeserializeProgramConfigData,
    #[error("Failed to deserialize proposal config account")]
    FailedToDeserializeProposalConfigData,
    #[error("Error on getting latest block hash")]
    ErrorOnGettingLatestBlockHash,
    #[error("Proposal status is not Approved")]
    ProposalStatusIsNotApproved
}

impl From<InvestorMultisigError> for ProgramError {
    fn from(e: InvestorMultisigError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

#[derive(Error, Debug, Copy, Clone)]
pub enum BusinessAnalystMultisigError {
    #[error("Failed to fetch program config account")]
    FailedToFetchProgramConfigAccount,
    #[error("Failed to fetch multisig config account")]
    FailedToFetchMultisigConfigAccount,
    #[error("Failed to fetch proposal config account")]
    FailedToFetchProposalConfigAccount,
    #[error("Failed to deserialize multisig config data")]
    FailedToDeserializeMultisigConfigData,
    #[error("Failed to deserialize program config account")]
    FailedToDeserializeProgramConfigData,
    #[error("Failed to deserialize proposal config account")]
    FailedToDeserializeProposalConfigData,
    #[error("Failed to build vault_transaction_execute instruction")]
    FailedToBuildVaultTransactionExecuteInstruction,
    #[error("Error on getting latest block hash")]
    ErrorOnGettingLatestBlockHash,
    #[error("Proposal status is not Approved")]
    ProposalStatusIsNotApproved
}

impl From<BusinessAnalystMultisigError> for ProgramError {
    fn from(e: BusinessAnalystMultisigError) -> Self {
        ProgramError::Custom(e as u32)
    }
}