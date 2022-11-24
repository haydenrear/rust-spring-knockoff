#![feature(decl_macro)]

use async_std::task as async_task;
use bson::{doc, Bson, Document};
use data_framework::{Entity, HDatabase, Repo, RepoDelegate};
use lazy_static::lazy_static;
use mongodb::options::{
    ClientOptions, FindOptions, InsertOneOptions, ResolverConfig, WriteConcern,
};
use mongodb::results::DatabaseSpecification;
use mongodb::{Client, Cursor, Database};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::borrow::{Borrow, BorrowMut};
use std::cell::Cell;
use std::collections::LinkedList;
use std::error::Error;
use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};
use std::task;

use async_recursion::async_recursion;
use async_trait::async_trait;

trait DbTrait: Send + Sync {}

impl DbTrait for Db {}

pub struct Db {
    pub client_options: Mutex<Option<ClientOptions>>,
    pub client_uri: &'static str,
}

lazy_static! {
    static ref DB: Arc<Db> = Arc::new(Db {
        client_options: Mutex::new(None),
        client_uri: "mongodb://admin:admin@localhost:27017/?authSource=admin"
    });
}

macro foo($i: item) {
    $i
}

foo!(pub struct A;);

#[test]
fn test() {
    let a = A{};
}

pub struct MongoRepo(&'static Db, &'static str, &'static str);

impl MongoRepo {
    #[async_recursion]
    pub async fn find_next<T: Entity<String> + Serialize + for<'de> Deserialize<'de> + Send + Sync>(
        &self,
        mut lst: LinkedList<T>,
        mut crsr: Cursor<T>,
    ) -> LinkedList<T> {
        let this_bson = Bson::Document(Document::try_from(crsr.current()).unwrap());
        lst.push_back(bson::from_bson(this_bson).unwrap());
        if crsr.advance().await.unwrap() {
            return self.find_next(lst, crsr).await;
        }
        lst
    }
    pub fn new(collection: &'static str, database: &'static str) -> Box<MongoRepo> {
        Box::new(MongoRepo(&DB, collection, database))
    }

}

#[async_trait]
impl<'a, T: Entity<String> + Serialize + for<'de> Deserialize<'de> + Send + Sync>
Repo<'a, T, String> for MongoRepo {
    type Data = &'static String;

    async fn find_all(&self) -> LinkedList<T> {
        let f = self
            .0
            .get_connection_from()
            .database(self.2)
            .collection(self.1)
            .find(None, None)
            .await
            .unwrap();
        let mut to_return: LinkedList<T> = LinkedList::<T>::new();
        self.find_next(to_return, f).await
    }

    async fn find_by_id(&self, id: String) -> Option<T> {
        let found = self
            .0
            .get_connection_from()
            .database(self.2)
            .collection(self.1)
            .find_one(
                doc! {
                    "id": id
                },
                None,
            )
            .await.unwrap_or(None);
        let f: Option<T> = found.or(None)
            .map(|d| {
                bson::from_bson::<T>(Bson::Document(d))
                    .ok()
            })
            .flatten();
        f
    }

    async fn save(&self, to_save: &'a T) -> String {
        self.0
            .get_connection_from()
            .database(self.2)
            .collection::<T>(self.1)
            .insert_one(to_save, None)
            .await
            .unwrap()
            .inserted_id
            .to_string()
            .clone()
    }

    fn get(data: Option<Self::Data>) -> Self
        where
            Self: Sized,
    {
        MongoRepo(&DB, data.unwrap().as_str(), data.unwrap().as_str())
    }
}

#[async_trait]
impl HDatabase<String> for Db {
    type DbId = &'static String;
    type DbConnection = Client;
    type DbOptions = ClientOptions;
    type RepoOption = &'static String;

    async fn list_database(&self) -> Vec<String> {
        self.get_databases().await
    }

    async fn get_repo<T>(
        &self,
        name: Option<&'static String>,
    ) -> Box<dyn Repo<T, String, Data = &'static String>>
        where
            T: Entity<String> + Serialize + for<'de> Deserialize<'de> + Send + Sync,
    {
        MongoRepo::new(name.unwrap().as_str(), name.unwrap().as_str())
    }

    async fn get_connection(&self, opts: Option<ClientOptions>) -> Client {
        match opts {
            Some(opt) => Client::with_options(opt).unwrap(),
            None => {
                Client::with_options(Db::default_options(self.client_uri).await).unwrap()
            }
        }
    }
}

impl Db {
    fn get_connection_from(&self) -> Client {
        async_task::block_on(self.get_connection(None))
    }

    fn set_client_options(&self, new_client_options: ClientOptions) {
        let mut client_options = self.client_options.lock().expect("");
        *client_options = Some(new_client_options.clone());
    }

    async fn default_options(client_uri: &str) -> ClientOptions {
        ClientOptions::parse_with_resolver_config(client_uri, ResolverConfig::cloudflare())
            .await
            .unwrap()
    }

    pub async fn get_databases(&self) -> Vec<String> {
        let result = self.get_connection_from().list_databases(None, None).await;
        result
            .unwrap()
            .iter()
            .map(|d: &DatabaseSpecification| d.name.clone())
            .collect::<Vec<String>>()
    }
}
