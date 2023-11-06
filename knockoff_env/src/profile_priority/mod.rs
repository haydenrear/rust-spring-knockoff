use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Copy, Default, Clone)]
pub struct Priority(usize);

type ProfileName = String;

#[derive(Serialize, Deserialize, Clone)]
pub struct EnvProfile(pub(crate) ProfileName, pub(crate) Priority);

impl Default for EnvProfile {
    fn default() -> Self {
        Self {
            0: "DefaultProfile".to_string(),
            1: Priority(0)
        }
    }
}

pub type EnvActiveProfileOrderings = Vec<EnvProfile>;

#[derive(Serialize, Deserialize, Clone)]
pub struct EnvironmentProfiles(EnvActiveProfileOrderings);

impl Default for EnvironmentProfiles {
    fn default() -> Self {
        Self {
            0: vec![
                EnvProfile::default()
            ],
        }
    }
}

