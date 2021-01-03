pub const MESSAGE_OK: bool = true;
pub const MESSAGE_ERROR: bool = false;
pub const MESSAGE_ROOM_DOESNT_EXIST: &str = "room doesn't exist / deleted";
pub const MESSAGE_ROOM_CREATED: &str = "room created";
pub const MESSAGE_ROOM_DELETED: &str = "room deleted";
pub const MESSAGE_USER_KICKED: &str = "user kicked";
pub const MESSAGE_FORBIDDEN_AUTHZ: &str = r#"{"action":"Forbidden","message":"you are not allowed to emit this message, closed automatically"}"#;
pub const FCM_API_TOKEN_KEY: &str = "okk";
pub const STUN_SERVER: &str = "stun://global.stun.twilio.com:3478?transport=udp";
pub const TURN_SERVER: &str = "turn://be72542c837be11bce6c2dcf60616a769db2eca3cc8ce82efc49d7df6e3aa07a:lAQi2fBmxzjXotzpapqJCjzBBrf3ItGwrztAPdb8tPU=@global.turn.twilio.com:3478?transport=udp";
// pub const TURN_SERVER2: &str = "turn://be72542c837be11bce6c2dcf60616a769db2eca3cc8ce82efc49d7df6e3aa07a:lAQi2fBmxzjXotzpapqJCjzBBrf3ItGwrztAPdb8tPU=@global.turn.twilio.com:3478?transport=tcp";
// pub const TURN_SERVER3: &str = "turn://be72542c837be11bce6c2dcf60616a769db2eca3cc8ce82efc49d7df6e3aa07a:lAQi2fBmxzjXotzpapqJCjzBBrf3ItGwrztAPdb8tPU=@global.turn.twilio.com:443?transport=tcp";

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
