use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum MessageSocket {
    Click { aya: String },
    RequestCorrection { uuid: String },
    ListRoom { uuid: String }
}