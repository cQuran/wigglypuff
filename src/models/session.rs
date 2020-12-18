use crate::{
    constants,
    models::{
        message::{MessageSocket, UserStatus},
        room::{Connect, Message, Room, Broadcast},
    },
    service::message_websocket,
};

use actix::{Actor, ActorContext, Addr, AsyncContext, Handler, Running, StreamHandler};
use actix_web_actors::ws;

pub struct Session {
    pub room_name: String,
    pub uuid: String,
    pub address: Addr<Room>,
    pub master_uuid: String,
}

impl Actor for Session {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, context: &mut Self::Context) {
        let address = context.address();
        self.address.do_send(Connect {
            room_name: self.room_name.to_owned(),
            uuid: self.uuid.to_owned(),
            address: address.recipient(),
        });
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        let user_disconnected_json_message = serde_json::to_string(&UserStatus {
            action: "UserLeave".to_string(),
            uuid: self.uuid.clone(),
        })
        .unwrap();
        self.address.do_send(Broadcast {
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
                        MessageSocket::MuteAllUser { .. }
                        | MessageSocket::MuteUser { .. }
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
                            if &self.master_uuid != uuid {
                                message_websocket::send_to_master(self, &message)
                            } else {
                                context.text(constants::MESSAGE_FORBIDDEN_AUTHZ.to_string());
                                context.stop();
                            }
                        }
                        MessageSocket::SignallingOfferSDP { ref uuid, .. }
                        | MessageSocket::SignallingAnswerSDP { ref uuid, .. }
                        | MessageSocket::SignallingCandidate { ref uuid, .. }
                        | MessageSocket::Leave { ref uuid, .. } => {
                            if &self.uuid == uuid {
                                message_websocket::broadcast_to_room(self, &message);
                            } else {
                                context.text(constants::MESSAGE_FORBIDDEN_AUTHZ.to_string());
                                context.stop();
                            }
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
