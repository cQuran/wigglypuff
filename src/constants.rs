pub const MESSAGE_OK: bool = true;
pub const MESSAGE_ERROR: bool = false;
pub const MESSAGE_ROOM_DOESNT_EXIST: &str = "room doesn't exist / deleted";
pub const MESSAGE_ROOM_CREATED: &str = "room created";
pub const MESSAGE_ROOM_DELETED: &str = "room deleted";
pub const MESSAGE_USER_KICKED: &str = "user kicked";
pub const MESSAGE_FORBIDDEN_AUTHZ: &str = r#"{"action":"Forbidden","message":"you are not allowed to emit this message, closed automatically"}"#;
pub const FCM_API_TOKEN_KEY: &str = "okk";
pub const STUN_SERVER: &str = "stun://global.stun.twilio.com:3478?transport=udp";
pub const TURN_CREDENTIAL: &str = "e994ab564e859690d6e325ae7b2e08b0b42ac468836921fc7db5ebb2d080958c";
pub const TURN_USERNAME: &str = "e994ab564e859690d6e325ae7b2e08b0b42ac468836921fc7db5ebb2d080958c";
pub const TURN_ADDRESS: [&str; 3] = [
    "turn://global.turn.twilio.com:3478?transport=udp",
    "turn://global.turn.twilio.com:3478?transport=tcp",
    "turn://global.turn.twilio.com:443?transport=tcp",
];

pub const PLUGIN_WEBRTC: [&str; 10] = [
    "videotestsrc",
    "videoconvert",
    "autodetect",
    "vpx",
    "webrtc",
    "nice",
    "dtls",
    "srtp",
    "rtpmanager",
    "rtp",
];
