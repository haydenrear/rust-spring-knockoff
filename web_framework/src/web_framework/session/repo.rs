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