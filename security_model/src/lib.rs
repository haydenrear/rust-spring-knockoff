use std::collections::{HashMap, LinkedList};
use serde::{Serialize,Deserialize};

pub trait User: Send + Sync + Copy {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserSession {
    data: SessionData,
    id: u64
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SessionData {
    session_data: HashMap<String, String>
}

impl Default for SessionData {
    fn default() -> Self {
        Self {
            session_data: HashMap::new()
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
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
