use serde::{Deserialize, Serialize};

use super::chat_message::ChatMessage;

#[derive(Serialize, Deserialize)]
pub enum MessageType {
    NewMessage,
    UserList,
}

#[derive(Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub message_type: MessageType,
    pub message: Option<ChatMessage>,
    pub users: Option<Vec<String>>,
}
