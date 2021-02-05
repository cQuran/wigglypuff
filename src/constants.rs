pub const MESSAGE_ROOM_DOESNT_EXIST: &str = "room doesn't exist / deleted";
pub const MESSAGE_USER_NOT_WEBSOCKET: &str = "please use it using websocket!";
pub const MESSAGE_ROOM_CREATED: &str = "room created";
pub const MESSAGE_ROOM_DELETED: &str = "room deleted";
pub const MESSAGE_JSON_PARSE_ERROR: &str = "error parsing json";
pub const MESSAGE_USER_KICKED: &str = "user kicked";
pub const MESSAGE_FORBIDDEN_AUTHZ: &str = r#"{"action":"Forbidden","message":"you are not allowed to emit this message, closed automatically"}"#;

pub const SENTRY_TOKEN: &str = "https://36cfa77ede8e493db712081b64b07d36@o512685.ingest.sentry.io/5613343";
pub const STUN_SERVER: &str = "stun://global.stun.twilio.com:3478?transport=udp";
pub const TURN_SERVER: &str = "turn://628db864f24ce4b571684ff1bd8a6551675007b98790aedfe75c818837d48bc9:aGW9tjvyPDe2PuxY47GseEo5QyZIWFzgPB31bLlXy/I=@global.turn.twilio.com:3478?transport=udp";
pub const TURN_SERVER_2: &str = "turn://628db864f24ce4b571684ff1bd8a6551675007b98790aedfe75c818837d48bc9:aGW9tjvyPDe2PuxY47GseEo5QyZIWFzgPB31bLlXy/I=@global.turn.twilio.com:3478?transport=tcp";
pub const TURN_SERVER_3: &str = "turn://628db864f24ce4b571684ff1bd8a6551675007b98790aedfe75c818837d48bc9:aGW9tjvyPDe2PuxY47GseEo5QyZIWFzgPB31bLlXy/I=@global.turn.twilio.com:443?transport=tcp";

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
