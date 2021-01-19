use crate::{
    constants, models,
    models::message_websocket::{MessageSocketType, UserStatus},
    service,
    service::{message_websocket, webrtc},
};

use actix::{Actor, ActorContext, Addr, AsyncContext, Handler, Running, StreamHandler};
use actix_web_actors::ws;

pub struct Session {
    pub room_name: String,
    pub uuid: String,
    pub room_address: Addr<service::room::Room>,
    pub master_uuid: String,
    pub webrtc_address: Addr<webrtc::supervisor::Supervisor>,
}

impl Actor for Session {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, context: &mut Self::Context) {
        let session_address = context.address();
        self.room_address.do_send(models::room::Connect {
            room_name: self.room_name.to_owned(),
            uuid: self.uuid.to_owned(),
            session_address: session_address.recipient(),
        });
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        let user_disconnected_json_message = serde_json::to_string(&UserStatus {
            action: "UserLeave",
            uuid: &self.uuid,
        })
        .unwrap();

        self.webrtc_address.do_send(models::supervisor::DeleteUser {
            uuid: self.uuid.clone(),
            room_name: self.room_name.clone(),
        });

        self.room_address.do_send(models::room::Broadcast {
            uuid: self.uuid.to_owned(),
            room_name: self.room_name.to_owned(),
            message: user_disconnected_json_message,
        });

        Running::Stop
    }
}

impl Handler<models::room::Message> for Session {
    type Result = ();

    fn handle(&mut self, message: models::room::Message, context: &mut Self::Context) {
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
                        MessageSocketType::MuteUser { ref uuid, .. } => {
                            if self.uuid == self.master_uuid || &self.uuid == uuid {
                                message_websocket::broadcast_to_room(self, &message);
                            } else {
                                context.text(constants::MESSAGE_FORBIDDEN_AUTHZ.to_string());
                                context.stop();
                            }
                        }
                        MessageSocketType::MuteAllUser { .. }
                        | MessageSocketType::AnswerCorrection { .. }
                        | MessageSocketType::MoveSura { .. }
                        | MessageSocketType::ClickAya { .. } => {
                            if self.uuid == self.master_uuid {
                                message_websocket::broadcast_to_room(self, &message);
                            } else {
                                context.text(constants::MESSAGE_FORBIDDEN_AUTHZ.to_string());
                                context.stop();
                            }
                        }
                        MessageSocketType::OfferCorrection { ref uuid, .. } => {
                            if &self.uuid != uuid {
                                message_websocket::send_to_master(self, &message)
                            } else {
                                context.text(constants::MESSAGE_FORBIDDEN_AUTHZ.to_string());
                                context.stop();
                            }
                        }
                        MessageSocketType::ICECandidate { .. } => {
                            message_websocket::send_to_client_webrtc(self, &message);
                        }
                        MessageSocketType::SessionDescription { .. } => {
                            message_websocket::send_to_client_webrtc(self, &message);
                        }
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
