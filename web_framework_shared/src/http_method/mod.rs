use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum HttpMethod {
    Get, Options, Head, Post, Put, Patch, Delete
}

impl Default for HttpMethod {
    fn default() -> Self {
        HttpMethod::Get
    }
}