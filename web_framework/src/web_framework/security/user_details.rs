use knockoff_security::knockoff_security::user_request_account::UserAccount;
use serde::{Deserialize, Serialize};
use data_framework::Repo;
use std::marker::PhantomData;
use std::any::Any;
use std::collections::HashMap;
use std::hash::Hash;

pub trait UserDetailsService<U, ID>: Send + Sync
    where
        U: UserAccount + Serialize + for<'a> Deserialize<'a> + Send + Sync
{
    async fn load_by_username(&self, id: &ID) -> Option<U>;
}

pub struct PersistenceUserDetailsService<'a, R, U>
    where
        U: UserAccount + Serialize + for<'de> Deserialize<'de> + Send + Sync,
        R: Repo<'a, U, String> {
    pub p: PhantomData<&'a (dyn Any + Send + Sync)>,
    pub u: PhantomData<U>,
    pub repo: Box<R>,
}

impl <'a, R, U> UserDetailsService<U, String> for PersistenceUserDetailsService<'a, R, U>
    where
        U: UserAccount + Serialize + for<'de> Deserialize<'de> + Send + Sync,
        R: Repo<'a, U, String> {
    async fn load_by_username(&self, id: &String) -> Option<U> {
        self.repo.find_by_id(id).await.clone()
    }
}

pub struct InMemoryUserDetailsService<U, ID>
    where
        U: UserAccount + Serialize + for<'a> Deserialize<'a> + Send + Sync,
        ID: Send + Sync
{
    user_accounts: HashMap<ID, U>
}

impl <U, ID> UserDetailsService<U,ID> for InMemoryUserDetailsService<U,ID>
    where
        U: UserAccount + Serialize + for<'a> Deserialize<'a> + Send + Sync,
        ID: Send + Sync + Eq + Hash
{
    async fn load_by_username(&self, id: &ID) -> Option<U> {
        self.user_accounts.get(id).cloned()
    }
}