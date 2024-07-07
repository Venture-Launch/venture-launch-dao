use solana_sdk::{
    pubkey::Pubkey,
    signature::Keypair,
};
use squads_multisig::solana_client::nonblocking::rpc_client::RpcClient;

pub struct BaseMultisigCreateArgs {
    pub rpc_client: RpcClient,
    pub multisig_create_keypair: Keypair,
    pub creator: Pubkey
}

pub struct BaseMultisigInitArgs {
    pub rpc_client: RpcClient,
    pub multisig_pda: Pubkey,
    pub creator: Pubkey
}

pub struct BaseMultisig {
    pub rpc_client: RpcClient,
    pub multisig_create_keypair: Option<Keypair>,
    pub creator: Pubkey,
    pub multisig_pda: Pubkey,
    pub vault_pda: Pubkey,
    pub program_config_pda: Pubkey,
    pub treasury: Pubkey
}