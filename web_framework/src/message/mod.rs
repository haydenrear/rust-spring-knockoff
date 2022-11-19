use serde::Serialize;

#[derive(Clone, Copy)]
pub struct MessageType<T: Serialize> {
    pub message: Option<T>
}


