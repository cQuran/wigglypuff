use crate::models::{
    message::MessageSocket,
    room::{Broadcast, Connect, Message, Room},
};
use actix::{Actor, Addr, AsyncContext, Handler, StreamHandler};
use actix_web_actors::ws;
use serde::Serialize;

#[derive(Serialize)]
pub struct Key {
    pub key: String,
}

pub struct Session {
    pub name: String,
    pub address: Addr<Room>,
}

impl Actor for Session {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, context: &mut Self::Context) {
        let address = context.address();
        self.address.do_send(Connect {
            address: address.recipient(),
        });
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
                let message_socket: MessageSocket = serde_json::from_str(text.as_str()).unwrap();
                match message_socket {
                    MessageSocket::Click { aya } => {
                        self.address.do_send(Broadcast {
                            room: "ok".to_string(),
                            message: text,
                        });
                    }
                    MessageSocket::RequestCorrection { uuid } => {}
                    MessageSocket::ListRoom { uuid } => {}
                }
            }
            _ => (),
        }
    }
}
