use serde::{Deserialize, Serialize};
use std::collections::linked_list::LinkedList;
use std::collections::HashMap;
use data_framework::Entity;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UserSession {
    pub data: SessionData,
    pub id: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SessionData {
    session_data: HashMap<String, String>,
}

pub trait UserAccount: Entity<String> + Clone {
    fn get_account_data(&self) -> AccountData;
    fn login(&self);
    fn get_password(&self) -> String;
}

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub struct AccountData {
    pub user_sessions: Vec<UserSession>,
    pub id: String
}
