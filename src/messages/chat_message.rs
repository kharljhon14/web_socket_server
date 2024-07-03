use std::collections::HashMap;

use chrono::{NaiveDateTime, Utc};
use rocket::{
    futures::{stream::SplitSink, SinkExt},
    tokio::sync::Mutex,
    State,
};
use rocket_ws::{stream::DuplexStream, Message};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::websocket_message::{MessageType, WebSocketMessage};

#[derive(Default)]
pub struct ChatRoom {
    connections: Mutex<HashMap<usize, ChatRoomConnection>>,
}

#[derive(Serialize, Deserialize)]
pub struct ChatMessage {
    pub messsage: String,
    pub author: String,
    pub created_at: NaiveDateTime,
}

pub struct ChatRoomConnection {
    username: String,
    sink: SplitSink<DuplexStream, Message>,
}

impl ChatRoom {
    pub async fn add(&self, id: usize, username: String, sink: SplitSink<DuplexStream, Message>) {
        let result = {
            let mut connections = self.connections.lock().await;

            let chat_message = ChatMessage {
                messsage: format!("{} joins the chat", username),
                author: "System".to_string(),
                created_at: Utc::now().naive_utc(),
            };

            let connection = ChatRoomConnection { username, sink };

            connections.insert(id, connection);

            Some(chat_message)
        };

        if let Some(message) = result {
            self.broadcast_message(message).await;
        }
    }

    pub async fn broadcast_message(&self, chat_message: ChatMessage) {
        let mut connections = self.connections.lock().await;

        let websocket_message = WebSocketMessage {
            message_type: MessageType::NewMessage,
            message: Some(chat_message),
            username: None,
            users: None,
        };

        for (_id, connection) in connections.iter_mut() {
            let _ = connection
                .sink
                .send(Message::text(json!(websocket_message).to_string()))
                .await;
        }
    }

    pub async fn broadcast_users(&self) {
        let mut connections = self.connections.lock().await;

        let users = connections
            .iter()
            .map(|(_id, connection)| connection.username.to_owned())
            .collect::<Vec<String>>();

        let websocket_message = WebSocketMessage {
            message_type: MessageType::UserList,
            message: None,
            username: None,
            users: Some(users),
        };

        for (_id, connection) in connections.iter_mut() {
            let _ = connection
                .sink
                .send(Message::Text(json!(websocket_message).to_string()))
                .await;
        }
    }
}

pub async fn handle_incoming_message(
    message_contents: Message,
    state: &State<ChatRoom>,
    sink: Option<SplitSink<DuplexStream, Message>>,
    connection_id: usize,
) {
    match message_contents {
        Message::Text(json) => {
            if let Ok(websocket_message) = serde_json::from_str::<WebSocketMessage>(&json) {
                match websocket_message.message_type {
                    MessageType::NewMessage => {
                        if let Some(message) = websocket_message.message {
                            state.broadcast_message(message).await;
                        }
                    }
                    MessageType::NewUser => {
                        let username = match websocket_message.username {
                            Some(username) => username,
                            None => return,
                        };

                        let ws_sink = match sink {
                            Some(ws_sink) => ws_sink,
                            None => return,
                        };

                        state.add(connection_id, username, ws_sink).await;
                        state.broadcast_users().await;
                    }
                    _ => {}
                }
            }
        }
        _ => {
            //  Unsupported
        }
    }
}
