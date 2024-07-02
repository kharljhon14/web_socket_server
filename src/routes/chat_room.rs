use std::collections::HashMap;

use chrono::{NaiveDateTime, Utc};
use rocket::{
    futures::{lock::Mutex, stream::SplitSink, SinkExt, StreamExt},
    State,
};
use rocket_ws::{stream::DuplexStream, Channel, Message, WebSocket};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Default)]
pub struct ChatRoom {
    connections: Mutex<HashMap<usize, SplitSink<DuplexStream, Message>>>,
}

#[derive(Serialize, Deserialize)]
struct ChatMessage {
    messsage: String,
    author: String,
    created_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize)]
enum MessageType {
    NewMessage,
    UserList,
}

#[derive(Serialize, Deserialize)]
struct WebSocketMessage {
    message_type: MessageType,
    message: Option<ChatMessage>,
    users: Option<Vec<String>>,
}

impl ChatRoom {
    async fn add(&self, id: usize, sink: SplitSink<DuplexStream, Message>) {
        let result = {
            let mut connections = self.connections.lock().await;

            let chat_message = ChatMessage {
                messsage: format!("New user joins"),
                author: "System".to_string(),
                created_at: Utc::now().naive_utc(),
            };

            connections.insert(id, sink);

            Some(chat_message)
        };

        if let Some(message) = result {
            self.broadcast_message(message).await;
        }
    }

    async fn broadcast_message(&self, chat_message: ChatMessage) {
        let mut connections = self.connections.lock().await;

        let websocket_message = WebSocketMessage {
            message_type: MessageType::NewMessage,
            message: Some(chat_message),
            users: None,
        };

        for (_id, connection) in connections.iter_mut() {
            let _ = connection
                .send(Message::text(json!(websocket_message).to_string()))
                .await;
        }
    }

    async fn broadcast_users(&self) {
        let mut connections = self.connections.lock().await;

        let users = connections
            .iter()
            .map(|(id, _connection)| id.to_string())
            .collect::<Vec<String>>();
        let websocket_message = WebSocketMessage {
            message_type: MessageType::UserList,
            message: None,
            users: Some(users),
        };

        for (_id, connection) in connections.iter_mut() {
            let _ = connection
                .send(Message::Text(json!(websocket_message).to_string()))
                .await;
        }
    }
}

async fn handle_incoming_message(
    message_contents: Message,
    state: &State<ChatRoom>,
    _connection_id: usize,
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
                    _ => {}
                }
            }
        }
        _ => {
            //  Unsupported
        }
    }
}

#[rocket::get("/")]
pub async fn chat<'r>(ws: WebSocket, state: &'r State<ChatRoom>) -> Channel<'r> {
    ws.channel(move |stream| {
        Box::pin(async move {
            let (ws_sink, mut ws_stream) = stream.split();
            let connection_id = rand::random::<usize>();
            state.add(connection_id, ws_sink).await;
            state.broadcast_users().await;

            while let Some(message) = ws_stream.next().await {
                if let Ok(message_contents) = message {
                    handle_incoming_message(message_contents, state, connection_id).await;
                }
            }
            Ok(())
        })
    })
}
