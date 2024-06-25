use thiserror::Error;
use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum BaseMultisigError {
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

impl From<BaseMultisigError> for ProgramError {
    fn from(e: BaseMultisigError) -> Self {
        ProgramError::Custom(e as u32)
    }
}