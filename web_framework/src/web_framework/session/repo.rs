use crate::web_framework::session::session::{HttpSession, WebApplication};
use data_framework::{Entity, Repo, RepoDelegate};
use lazy_static::lazy_static;
use mongo_repo::Db;
use mongo_repo::MongoRepo;
use knockoff_security::knockoff_security::user_request_account::SessionData;
use std::collections::LinkedList;
use std::sync::{Arc, Mutex};

use tokio_test::block_on;

use async_trait::async_trait;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub struct HttpSessionDelegate(HttpSession);
pub struct HttpSessionRepo;

lazy_static! {
    static ref DB: Arc<Db> = Arc::new(Db {
        client_options: Mutex::new(None),
        client_uri: "mongodb://admin:admin@localhost:27017/?authSource=admin"
    });
}

#[test]
fn test_http_session_repo() {
    tokio_test::block_on(test_insert_save())
}

async fn test_insert_save() {
    let http_session_repo = MongoRepo::new("http_session", "http_session");
    let to_save = HttpSession::new(
        String::from("10"),
        None,
        WebApplication::default(),
        SessionData::default(),
    );
    let saved_id = http_session_repo.save(&to_save).await;
    println!("{} is to save", to_save.get_id().unwrap().clone());
    println!("{} is id", saved_id.clone());
    let found: HttpSession = http_session_repo
        .find_by_id(String::from("10"))
        .await
        .unwrap();
    assert_eq!(found.get_id().unwrap(), String::from("10"));
}

#[test]
fn test_repo_delegate() {
    println!("hello")
}
