use thiserror::Error;

#[derive(Error, Debug, Copy, Clone)]
pub enum BrokerError {
    #[error("Failed to fetch program config account")]
    FailedToFetchProgramConfigAccount,
}