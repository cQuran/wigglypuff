use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct FCM<'a> {
    pub token: &'a str,
}
