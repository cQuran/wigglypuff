use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct UserStatus {
    pub action: String,
    pub uuid: String,
}

#[derive(PartialEq, Serialize, Deserialize, Clone)]
#[serde(tag = "action")]
pub enum MessageSocket {
    RequestAllOfferSDP {},
    SignallingOfferSDP {
        uuid: String,
        into: String,
        value: String,
    },
    SignallingAnswerSDP {
        uuid: String,
        into: String,
        value: String,
    },
    SignallingCandidate {
        uuid: String,
        into: String,
        value: String,
    },
    ClickAya {
        aya: i32,
    },
    Leave {
        uuid: String,
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
        id_quran: String,
    },
}
