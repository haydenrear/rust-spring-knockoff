use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum HttpMethod {
    Post,
    Get,
}

impl Default for HttpMethod {
    fn default() -> Self {
        HttpMethod::Get
    }
}