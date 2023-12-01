use std::string::ToString;
use axum::Extension;
use axum::response::{IntoResponse, Json};
use katana_core::accounts::{Account, DevAccountGenerator};
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::server_state::ServerState;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SerializedAccount {
    pub public_key: String,
    pub private_key: String,
    pub address: String,
    pub class_hash: String
}

impl From<Account> for SerializedAccount {
    fn from(value: Account) -> Self {
        let account = json!(value);
        SerializedAccount {
            public_key: value.public_key.to_string(),
            private_key: value.private_key.to_string(),
            address: value.address.to_string(),
            class_hash: value.class_hash.to_string()
        }
    }
}

fn parse_seed(seed: &str) -> [u8; 32] {
    let seed = seed.as_bytes();

    if seed.len() >= 32 {
        unsafe { *(seed[..32].as_ptr() as *const [u8; 32]) }
    } else {
        let mut actual_seed = [0u8; 32];
        seed.iter().enumerate().for_each(|(i, b)| actual_seed[i] = *b);
        actual_seed
    }
}

fn generate_accounts(seed: &str, total_accounts: u8) -> Vec<Account> {
    DevAccountGenerator::new(total_accounts)
        .with_seed( parse_seed(seed))
        .generate()
}

pub fn get_serialized_accounts(seed: &str, total_accounts: u8) -> Vec<SerializedAccount> {

    let accounts = generate_accounts(seed, total_accounts);

    accounts.iter().map(|account| account.clone().into()).collect()
}

pub async fn handler(Extension(state): Extension<ServerState>) -> impl IntoResponse {
    Json(json!(generate_accounts(&state.starknet.seed, state.starknet.total_accounts)))
}
