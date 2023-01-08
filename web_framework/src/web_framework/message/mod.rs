use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize)]
pub struct MessageType<T: Serialize> where Self: 'static, T: 'static{
    pub message: Option<T>,
}