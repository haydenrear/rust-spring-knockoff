use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::LinkedList;

#[async_trait]
pub trait Repo<'a, T: Entity<ID>, ID> : Send + Sync{
    type Data;
    async fn find_all(&self) -> LinkedList<T>
    where
        Self: Sized;
    async fn find_by_id(&self, id: &ID) -> Option<T>
    where
        Self: Sized;
    async fn save(&self, to_save: &'a T) -> ID
    where
        Self: Sized;
    fn get(data: Option<Self::Data>) -> Self
    where
        Self: Sized;
}

#[async_trait]
pub trait RepoDelegate<'a, T: Entity<ID>, ID> {
    type REPO: Repo<'a, T, ID>;
    type ID;
    fn identifier() -> Self::ID;
    async fn find_all() -> LinkedList<T>;
    async fn find_by_id(id: &ID) -> Option<T>;
    async fn save(to_save: &'a T) -> ID;
}

pub trait Entity<ID>: Serialize + for<'de> Deserialize<'de> + Send + Sync {
    fn get_id(&self) -> Option<ID>;
    fn set_id(&mut self, id: ID);
}

#[async_trait]
pub trait HDatabase<ID>: Send + Sync {
    type DbId;
    type DbConnection;
    type DbOptions;
    type RepoOption;
    async fn list_database(&self) -> Vec<String>;
    async fn get_connection(&self, opts: Option<Self::DbOptions>) -> Self::DbConnection;
    async fn get_repo<T>(
        &self,
        name: Option<Self::DbId>,
    ) -> Box<dyn Repo<T, ID, Data = Self::RepoOption>>
    where
        T: Entity<ID>;
}
