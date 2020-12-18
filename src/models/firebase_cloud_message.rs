use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct FCM {
    pub token: String,
}