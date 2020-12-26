use crate::{
    constants,
    models::{
        message::{MessageSocket, UserStatus},
        room::{Broadcast, Connect, Message, Room},
        webrtc::WebRTC,
    },
    service::message_websocket,
};

use actix::{Actor, ActorContext, Addr, AsyncContext, Handler, Running, StreamHandler};
use actix_web_actors::ws;

pub struct Session {
    pub room_name: String,
    pub uuid: String,
    pub room_address: Addr<Room>,
    pub master_uuid: String,
    pub webrtc_address: Addr<WebRTC>,
}

impl Actor for Session {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, context: &mut Self::Context) {
        let room_address = context.address();
        self.room_address.do_send(Connect {
            room_name: self.room_name.to_owned(),
            uuid: self.uuid.to_owned(),
            room_address: room_address.recipient(),
        });
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        let user_disconnected_json_message = serde_json::to_string(&UserStatus {
            action: "UserLeave".to_string(),
            uuid: self.uuid.clone(),
        })
        .unwrap();
        self.room_address.do_send(Broadcast {
            uuid: self.uuid.to_owned(),
            room_name: self.room_name.to_owned(),
            message: user_disconnected_json_message,
        });
        Running::Stop
    }
}

impl Handler<Message> for Session {
    type Result = ();

    fn handle(&mut self, message: Message, context: &mut Self::Context) {
        context.text(message.0);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Session {
    fn handle(
        &mut self,
        message: Result<ws::Message, ws::ProtocolError>,
        context: &mut Self::Context,
    ) {
        match message {
            Ok(ws::Message::Ping(message)) => context.pong(&message),
            Ok(ws::Message::Text(text)) => {
                let message_value = serde_json::from_str(text.as_str());
                match message_value {
                    Ok(message) => match message {
                        MessageSocket::MuteUser { ref uuid, .. } => {
                            if self.uuid == self.master_uuid || &self.uuid == uuid {
                                message_websocket::broadcast_to_room(self, &message);
                            } else {
                                context.text(constants::MESSAGE_FORBIDDEN_AUTHZ.to_string());
                                context.stop();
                            }
                        }
                        MessageSocket::MuteAllUser { .. }
                        | MessageSocket::AnswerCorrection { .. }
                        | MessageSocket::MoveSura { .. }
                        | MessageSocket::ClickAya { .. } => {
                            if self.uuid == self.master_uuid {
                                message_websocket::broadcast_to_room(self, &message);
                            } else {
                                context.text(constants::MESSAGE_FORBIDDEN_AUTHZ.to_string());
                                context.stop();
                            }
                        }
                        MessageSocket::OfferCorrection { ref uuid, .. } => {
                            if &self.uuid != uuid {
                                message_websocket::send_to_master(self, &message)
                            } else {
                                context.text(constants::MESSAGE_FORBIDDEN_AUTHZ.to_string());
                                context.stop();
                            }
                        }
                        MessageSocket::ICECandidate { .. } => {
                            message_websocket::send_to_client_webrtc(self, &message);
                        }
                        _ => message_websocket::broadcast_to_room(self, &message),
                    },
                    _ => {
                        context.text(constants::MESSAGE_FORBIDDEN_AUTHZ.to_string());
                        context.stop();
                    }
                }
            }
            _ => {
                context.text(constants::MESSAGE_FORBIDDEN_AUTHZ.to_string());
                context.stop()
            }
        }
    }
}
