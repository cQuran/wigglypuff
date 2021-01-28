pub const MESSAGE_ROOM_DOESNT_EXIST: &str = "room doesn't exist / deleted";
pub const MESSAGE_USER_NOT_WEBSOCKET: &str = "please use it using websocket!";
pub const MESSAGE_ROOM_CREATED: &str = "room created";
pub const MESSAGE_ROOM_DELETED: &str = "room deleted";
pub const MESSAGE_JSON_PARSE_ERROR: &str = "error parsing json";
pub const MESSAGE_USER_KICKED: &str = "user kicked";
pub const MESSAGE_FORBIDDEN_AUTHZ: &str = r#"{"action":"Forbidden","message":"you are not allowed to emit this message, closed automatically"}"#;

pub const FCM_API_TOKEN_KEY: &str = "okk";
pub const SENTRY_TOKEN: &str = "https://36cfa77ede8e493db712081b64b07d36@o512685.ingest.sentry.io/5613343";
pub const STUN_SERVER: &str = "stun://global.stun.twilio.com:3478?transport=udp";
pub const TURN_SERVER: &str = "turn://7780c64927a75734311ddfd5d49f4eeaf77fd3ac98f0bc39640ee826ec894c20:wpP2iRS6TS8t6w9pDrzV1voIx2nXh5M2WfEerY5J7bE=@global.turn.twilio.com:3478?transport=tcp";

pub const PLUGIN_WEBRTC: [&str; 8] = [
    "autodetect",
    "vpx",
    "webrtc",
    "nice",
    "dtls",
    "srtp",
    "rtpmanager",
    "rtp",
];
