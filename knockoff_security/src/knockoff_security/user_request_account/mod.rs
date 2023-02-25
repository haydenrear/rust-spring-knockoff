use serde::{Deserialize, Serialize};
use std::collections::linked_list::LinkedList;
use std::collections::HashMap;

pub trait User: Send + Sync + Copy {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserSession {
    data: SessionData,
    id: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SessionData {
    session_data: HashMap<String, String>,
}

impl Default for SessionData {
    fn default() -> Self {
        Self {
            session_data: HashMap::new(),
        }
    }
}

pub trait UserAccount {
    fn get_user_session(&self) -> Box<UserSession>;
    fn login(&self);
}

pub trait AccountData {
    fn get_user_sessions(&self) -> LinkedList<Box<UserSession>>;
    fn get_id(&self) -> u16;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
