use rocket::{futures::StreamExt, State};
use rocket_ws::{Channel, WebSocket};

use crate::messages::chat_message::{handle_incoming_message, ChatRoom};

#[rocket::get("/")]
pub async fn chat<'r>(ws: WebSocket, state: &'r State<ChatRoom>) -> Channel<'r> {
    ws.channel(move |stream| {
        Box::pin(async move {
            let (ws_sink, mut ws_stream) = stream.split();
            let connection_id = rand::random::<usize>();

            if let Some(message) = ws_stream.next().await {
                if let Ok(message_contents) = message {
                    handle_incoming_message(message_contents, state, Some(ws_sink), connection_id)
                        .await;
                }
            }

            while let Some(message) = ws_stream.next().await {
                if let Ok(message_contents) = message {
                    handle_incoming_message(message_contents, state, None, connection_id).await;
                }
            }
            Ok(())
        })
    })
}
