pub const MESSAGE_ROOM_DOESNT_EXIST: &str = "room doesn't exist / deleted";
pub const MESSAGE_USER_NOT_WEBSOCKET: &str = "please use it using websocket!";
pub const MESSAGE_ROOM_CREATED: &str = "room created";
pub const MESSAGE_ROOM_DELETED: &str = "room deleted";
pub const MESSAGE_NAT_REFRESHED: &str = "nat refreshed";
pub const MESSAGE_JSON_PARSE_ERROR: &str = "error parsing json";
pub const MESSAGE_USER_KICKED: &str = "user kicked";
pub const MESSAGE_FORBIDDEN_AUTHZ: &str = r#"{"action":"Forbidden","message":"you are not allowed to emit this message, closed automatically"}"#;

pub const SENTRY_TOKEN: &str = "https://36cfa77ede8e493db712081b64b07d36@o512685.ingest.sentry.io/5613343";
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
