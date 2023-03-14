use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GrantedAuthority {
    pub authority: String,
}

impl GrantedAuthority {
    pub fn get_authority(&self) -> &str {
        &self.authority
    }
}
