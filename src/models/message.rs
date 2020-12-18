use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct UserStatus {
    pub action: String,
    pub uuid: String,
}

#[derive(PartialEq, Serialize, Deserialize, Clone)]
#[serde(tag = "action")]
pub enum MessageSocket {
    SignallingOfferSDP {
        value: String,
    },
    SignallingAnswerSDP {
        value: String,
    },
    SignallingCandidate {
        value: String,
    },
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
}
