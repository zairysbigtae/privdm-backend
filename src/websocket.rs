use argon2::{password_hash::{rand_core::OsRng, SaltString}, Argon2, PasswordHash, PasswordHasher};
use axum::{extract::{ws::{self, Utf8Bytes, WebSocket}, State, WebSocketUpgrade}, response::Response};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Pool, Postgres};

pub async fn ws_handler(ws: WebSocketUpgrade, State(pool): State<PgPool>) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, pool))
}

async fn handle_socket(mut socket: WebSocket, pool: Pool<Postgres>) {
    while let Some(Ok(msg)) = socket.recv().await {
        let text = match msg {
            ws::Message::Text(t) => t,
            ws::Message::Close(_) => break,
            _ => continue,
        };

        let ws_msg = |txt: &str| ws::Message::Text(txt.into());

        match text.as_str() {
            "quit" => break,
            "help" => {
                socket.send(ws_msg(
                "
--- MESSAGE ---
get_msgs - Get messages
insert_msg - Sends a message
delete_msg - Deletes a message
edit_msg - Edits a message

--- USER ---
get_users - Get users
insert_user - Creates a new account
delete_user - Deletes a user
edit_user - Edits a user

--- ROOM ---
get_rooms - Get rooms
insert_room - Creates a new room
delete_room - Deletes a room
edit_room - Edits a room")).await.unwrap();
            }
            _ => ()
        }

        msgs_command(&mut socket, &pool, &text).await;
        users_command(&mut socket, &pool, &text).await;
        rooms_command(&mut socket, &pool, &text).await;
    }
}

async fn msgs_command(socket: &mut WebSocket, pool: &PgPool, text: &Utf8Bytes) {
    let ws_msg = |txt: &str| ws::Message::Text(txt.into());

    match text.as_str() {
        // 1. get messages
        // 2. insert message
        // 3. delete message
        // 4. edit message
        "get_msgs" => {
            socket.send(ws_msg("Requesting messages...")).await.unwrap();

            let record = sqlx::query!("SELECT id, content, room_id, user_id, sent_at FROM messages")
                .fetch_all(pool)
                .await.unwrap();

            if socket.send(ws_msg(&format!("Content: {record:?}"))).await.is_err() { return; }
        }
        "insert_msg" => {
            socket.send(ws_msg("Inserting a message...")).await.unwrap();
            socket.send(ws_msg("content: ")).await.unwrap();

            if let Some(Ok(content)) = socket.recv().await {
                let content_str = content.to_text().unwrap();
                println!("{content_str}");

                sqlx::query!("INSERT INTO messages (content) VALUES ($1)", content_str)
                    .execute(pool)
                    .await.unwrap();

                if socket.send(ws_msg(&format!("Inserted \"{content:?}\""))).await.is_err() { return; }
            }
        }
        "delete_msg" => {
            socket.send(ws_msg("Deleting a message...")).await.unwrap();
            socket.send(ws_msg("ID: ")).await.unwrap();

            if let Some(Ok(id)) = socket.recv().await {
                let id_str = id.to_text().unwrap();

                sqlx::query!("DELETE FROM messages WHERE id = $1", 
                    id_str.parse::<i32>().expect("Couldn't get the ID, is the id valid or not?"))
                    .execute(pool)
                    .await.unwrap();

                if socket.send(ws_msg("Deleted")).await.is_err() { return; }
            }
        }
        "edit_msg" => todo!(),
        _ => ()
    }
}

async fn users_command(socket: &mut WebSocket, pool: &PgPool, text: &Utf8Bytes) {
    let ws_msg = |txt: &str| ws::Message::Text(txt.into());

    match text.as_str() {
        // 1. get users
        // 2. insert user
        // 3. delete user
        // 4. edit user
        "get_users" => {
            socket.send(ws_msg("Requesting messages...")).await.unwrap();

            let record = sqlx::query!("SELECT id, room_id, name, joined_at FROM users")
                .fetch_all(pool)
                .await.unwrap();

            if socket.send(ws_msg(&format!("Users: {record:?}"))).await.is_err() { return; }
        }
        "insert_user" => {
            socket.send(ws_msg("Creating a new account...")).await.unwrap();
            socket.send(ws_msg("Account name: ")).await.unwrap();

            if let Some(Ok(name)) = socket.recv().await {
                socket.send(ws_msg("Pass: ")).await.unwrap();

                if let Some(Ok(pass)) = socket.recv().await {
                    let name_str = name.to_text().unwrap();
                    let pass_str = pass.to_text().unwrap();

                    println!("{name_str}");

                    let salt = SaltString::generate(&mut OsRng);
                    let argon2 = Argon2::default();

                    let hashed = 
                        argon2.hash_password(pass_str.as_bytes(), &salt)
                        .unwrap()
                        .to_string();

                    let parsed_hash = PasswordHash::new(&hashed).unwrap();

                    sqlx::query!("INSERT INTO users (name, pass_hash) VALUES ($1, $2)", name_str, parsed_hash.to_string())
                        .execute(pool)
                        .await.unwrap();

                    if socket.send(ws_msg(&format!("Inserted \"{name:?}\""))).await.is_err() { return; }
                }
            }
        }
        "delete_user" => {
            socket.send(ws_msg("Deleting a user...")).await.unwrap();
            socket.send(ws_msg("ID: ")).await.unwrap();

            if let Some(Ok(id)) = socket.recv().await {
                let id_str = id.to_text().unwrap();

                sqlx::query!("DELETE FROM users WHERE id = $1",
                    id_str.parse::<i32>().expect("Couldn't get the ID, is the id valid or not?"))
                    .execute(pool)
                    .await.unwrap();

                if socket.send(ws_msg("Deleted")).await.is_err() { return; }
            }
        }
        "edit_user" => todo!(),
        _ => ()
    }
}

async fn rooms_command(socket: &mut WebSocket, pool: &PgPool, text: &Utf8Bytes) {
    let ws_msg = |txt: &str| ws::Message::Text(txt.into());

    match text.as_str() {
        // 1. get users
        // 2. insert user
        // 3. delete user
        // 4. edit user
        "get_rooms" => {
            socket.send(ws_msg("Requesting rooms...")).await.unwrap();

            let record = sqlx::query!("SELECT id, created_at, expires_at, name FROM rooms")
                .fetch_all(pool)
                .await.unwrap();

            if socket.send(ws_msg(&format!("Rooms: {record:?}"))).await.is_err() { return; }
        }
        "insert_room" => {
            socket.send(ws_msg("Creating a new room...")).await.unwrap();
            socket.send(ws_msg("Room name: ")).await.unwrap();

            if let Some(Ok(name)) = socket.recv().await {
                let name_str = name.to_text().unwrap();
                println!("{name_str}");

                sqlx::query!("INSERT INTO rooms (name) VALUES ($1)", name_str)
                    .execute(pool)
                    .await.unwrap();

                if socket.send(ws_msg(&format!("Inserted \"{name:?}\""))).await.is_err() { return; }
            }
        }
        "delete_room" => {
            socket.send(ws_msg("Deleting a room...")).await.unwrap();
            socket.send(ws_msg("ID: ")).await.unwrap();

            if let Some(Ok(id)) = socket.recv().await {
                let id_str = id.to_text().unwrap();

                sqlx::query!("DELETE FROM rooms WHERE id = $1",
                    id_str.parse::<i32>().expect("Couldn't get the ID, is the id valid or not?"))
                    .execute(pool)
                    .await.unwrap();

                if socket.send(ws_msg("Deleted")).await.is_err() { return; }
            }
        }
        "edit_room" => todo!(),
        _ => ()
    }
}

