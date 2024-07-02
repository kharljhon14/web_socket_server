use messages::chat_message::ChatRoom;
use routes::chat_room::chat;

mod messages;
mod routes;

#[rocket::main]
async fn main() {
    let _ = rocket::build()
        .mount("/", rocket::routes![chat])
        .manage(ChatRoom::default())
        .launch()
        .await;
}
