use serde::{Deserialize, Serialize};
use crate::web_framework::security::security::AuthenticationToken;


#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SecurityContextHolder {
    pub auth_token: Option<AuthenticationToken>
}

