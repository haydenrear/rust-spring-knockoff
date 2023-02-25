pub trait PasswordEncoder : Send + Sync {
    fn encode_password(&self, unencoded: &str) -> String;
}

#[derive(Clone)]
pub struct NoOpPasswordEncoder;

impl PasswordEncoder for NoOpPasswordEncoder {
    fn encode_password(&self, unencoded: &str) -> String {
        unencoded.to_string()
    }
}

