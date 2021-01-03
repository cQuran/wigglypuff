use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct UserStatus<'a> {
    pub action: &'a str,
    pub uuid: &'a str,
}

#[derive(PartialEq, Serialize, Deserialize, Clone)]
#[serde(tag = "action")]
pub enum MessageSocketType {
    ClickAya {
        aya: i32,
    },
    OfferCorrection {
        uuid: String,
    },
    AnswerCorrection {
        uuid: String,
        result: bool,
    },
    MuteUser {
        uuid: String,
    },
    MuteAllUser {},
    MoveSura {
        id_quran: i32,
    },
    ICECandidate {
        candidate: String,
        #[serde(rename = "sdpMLineIndex")]
        sdp_mline_index: u32,
    },
    SessionDescription {
        #[serde(rename = "type")]
        types: String,
        sdp: String,
    }
}
