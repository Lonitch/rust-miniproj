use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

use crate::chat_server::ChatServer;
use crate::error::ChatError;
use crate::message::ChatMessage;

/// Generate a random room name
pub fn random_room_name() -> String {
    let mut rng = rand::thread_rng();
    format!("room-{}", rng.gen_range(1000..9999))
}

/// Generate a random user name
pub fn random_user_name() -> String {
    let mut rng = rand::thread_rng();
    format!("user-{}", rng.gen_range(1000..9999))
}

/// Generate a random user-reply message
pub fn random_message() -> String {
    let messages = [
        "Hello!",
        "Yes, what's up?",
        "I heard you calling me.",
        "Sure, I'm here.",
        "I'm responding to the bot.",
        "Random user is present. :-)",
    ];
    let mut rng = rand::thread_rng();
    messages.choose(&mut rng).unwrap().to_string()
}

/// Manages room creation and deletion
async fn manage_rooms(
    server: Arc<Mutex<ChatServer>>,
    rooms: Arc<Mutex<Vec<String>>>,
    users_in_room: Arc<Mutex<HashMap<String, Vec<String>>>>,
) {
    loop {
        let action = {
            // create a fresh RNG so we don't hold ThreadRng across .await
            // used only inside brace, and then dropped before hit any .await
            let mut rng = rand::thread_rng();
            rng.gen_range(0..5)
        };

        match action {
            0..=3 => {
                // CREATE ROOM
                let room = random_room_name();
                let mut srv = server.lock().await;
                let result = srv.create_room(room.clone()).await;
                drop(srv);

                match result {
                    Ok(_) => {
                        rooms.lock().await.push(room.clone());
                        users_in_room.lock().await.insert(room.clone(), Vec::new());

                        let bot_name = format!("{}-bot", &room);
                        let mut srv2 = server.lock().await;
                        if let Err(e) = srv2.join_room(&room, bot_name.clone()).await {
                            eprintln!("[ERROR]: adding bot '{}' to '{}': {:?}", bot_name, room, e);
                        } else {
                            println!("[LOG]: Created room '{}' + bot '{}'", room, bot_name);
                        }
                    }
                    Err(ChatError::RoomExists) => {
                        println!("[ERROR]: Room '{}' already exists!", room);
                    }
                    Err(e) => {
                        eprintln!("[ERROR]: creating room '{}': {:?}", room, e);
                    }
                }
            }
            _ => {
                // REMOVE ROOM
                let existing_rooms = rooms.lock().await.clone();
                if !existing_rooms.is_empty() {
                    let chosen_room = {
                        let mut rng = rand::thread_rng();
                        existing_rooms.choose(&mut rng).unwrap().clone()
                    };

                    {
                        let mut map = users_in_room.lock().await;
                        if let Some(u_list) = map.get_mut(&chosen_room) {
                            let mut srv = server.lock().await;
                            for user in u_list.clone() {
                                let _ = srv.leave_room(&chosen_room, &user).await;
                            }
                            u_list.clear();

                            let bot_name = format!("{}-bot", chosen_room);
                            let _ = srv.leave_room(&chosen_room, &bot_name).await;
                        }
                    }

                    let mut srv2 = server.lock().await;
                    match srv2.remove_room(&chosen_room).await {
                        Ok(_) => {
                            println!("[LOG]: Removed room '{}'", chosen_room);

                            // RULE OF THUMB FOR WRAPPING MUTEX LOCK:
                            // whenever you have multiple "obj.lock().await"
                            // in the same scope and returned objects require
                            // the same resource to do things, you need {}

                            {
                                let mut lock_rooms = rooms.lock().await;
                                lock_rooms.retain(|r| r != &chosen_room);
                            }
                            users_in_room.lock().await.remove(&chosen_room);
                        }
                        Err(ChatError::RoomNotEmpty) => {
                            println!("[ERROR]: Room '{}' not empty, cannot remove", chosen_room);
                        }
                        Err(e) => {
                            eprintln!("[ERROR]: removing room '{}': {:?}", chosen_room, e);
                        }
                    }
                }
            }
        }

        sleep(Duration::from_millis(1000)).await;
    }
}

/// Manages user joining and leaving rooms
async fn manage_users(
    server: Arc<Mutex<ChatServer>>,
    rooms: Arc<Mutex<Vec<String>>>,
    users_in_room: Arc<Mutex<HashMap<String, Vec<String>>>>,
) {
    loop {
        let action = {
            let mut rng = rand::thread_rng();
            rng.gen_range(0..4)
        };

        match action {
            0..=2 => {
                // ADD user
                let current_rooms = { rooms.lock().await.clone() };
                if !current_rooms.is_empty() {
                    let chosen_room = {
                        let mut rng = rand::thread_rng();
                        current_rooms.choose(&mut rng).unwrap().clone()
                    };

                    let new_user = random_user_name();
                    let mut srv = server.lock().await;
                    let result = srv.join_room(&chosen_room, new_user.clone()).await;
                    drop(srv);

                    if let Err(e) = result {
                        eprintln!(
                            "[ERROR]: Adding '{}' to '{}': {:?}",
                            new_user, chosen_room, e
                        );
                    } else {
                        println!("[LOG]: User '{}' joined room '{}'", new_user, chosen_room);
                        users_in_room
                            .lock()
                            .await
                            .entry(chosen_room)
                            .and_modify(|list| list.push(new_user));
                    }
                }
            }
            3 => {
                // REMOVE user
                let current_rooms = { rooms.lock().await.clone() };
                if !current_rooms.is_empty() {
                    let chosen_room = {
                        let mut rng = rand::thread_rng();
                        current_rooms.choose(&mut rng).unwrap().clone()
                    };

                    let mut map = users_in_room.lock().await;
                    if let Some(u_list) = map.get_mut(&chosen_room) {
                        if !u_list.is_empty() {
                            let idx = {
                                let mut rng = rand::thread_rng();
                                rng.gen_range(0..u_list.len())
                            };
                            let removed_user = u_list.remove(idx);
                            drop(map);

                            let mut srv = server.lock().await;
                            if let Err(e) = srv.leave_room(&chosen_room, &removed_user).await {
                                eprintln!(
                                    "[ERROR]: removing '{}' from '{}': {:?}",
                                    removed_user, chosen_room, e
                                );
                            } else {
                                println!(
                                    "[LOG]: User '{}' left room '{}'",
                                    removed_user, chosen_room
                                );
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        sleep(Duration::from_millis(1000)).await;
    }
}

/// Manages bot-user interactions in rooms
async fn manage_bot_interactions(
    server: Arc<Mutex<ChatServer>>,
    rooms: Arc<Mutex<Vec<String>>>,
    users_in_room: Arc<Mutex<HashMap<String, Vec<String>>>>,
) {
    loop {
        let current_rooms = { rooms.lock().await.clone() };
        let mut candidate_rooms = Vec::new();
        {
            let map = users_in_room.lock().await;
            for r in &current_rooms {
                if let Some(list) = map.get(r) {
                    if !list.is_empty() {
                        candidate_rooms.push(r.clone());
                    }
                }
            }
        }

        if !candidate_rooms.is_empty() {
            let chosen_room = {
                let mut rng = rand::thread_rng();
                candidate_rooms.choose(&mut rng).unwrap().clone()
            };

            let user_list = {
                let map = users_in_room.lock().await;
                map.get(&chosen_room).cloned().unwrap_or_default()
            };

            if !user_list.is_empty() {
                let user = {
                    let mut rng = rand::thread_rng();
                    user_list.choose(&mut rng).unwrap().clone()
                };
                let bot_name = format!("{}-bot", chosen_room);

                // bot calls the user
                let call = ChatMessage {
                    sender: bot_name.clone(),
                    room: chosen_room.clone(),
                    content: user.clone(),
                    timestamp: chrono::Utc::now(),
                };
                {
                    let srv = server.lock().await;
                    let _ = srv.send_message(&chosen_room, call).await;
                }
                println!(
                    "[LOG]: Bot '{}' in '{}' called user '{}'",
                    bot_name, chosen_room, user
                );

                // user reply
                let reply = ChatMessage {
                    sender: user.clone(),
                    room: chosen_room.clone(),
                    content: random_message(),
                    timestamp: chrono::Utc::now(),
                };
                {
                    let srv = server.lock().await;
                    let _ = srv.send_message(&chosen_room, reply).await;
                }
                println!("[LOG]: User '{}' replied in '{}'", user, chosen_room);
            }
        }

        sleep(Duration::from_millis(1000)).await;
    }
}

pub async fn run_simulation(duration: Duration) {
    let server = Arc::new(Mutex::new(ChatServer::new().await));
    let rooms = Arc::new(Mutex::new(Vec::<String>::new()));
    let users_in_room = Arc::new(Mutex::new(HashMap::<String, Vec<String>>::new()));

    // Spawn room management task
    {
        let server = Arc::clone(&server);
        let rooms = Arc::clone(&rooms);
        let users_in_room = Arc::clone(&users_in_room);
        tokio::spawn(manage_rooms(server, rooms, users_in_room));
    }

    // Spawn user management task
    {
        let server = Arc::clone(&server);
        let rooms = Arc::clone(&rooms);
        let users_in_room = Arc::clone(&users_in_room);
        tokio::spawn(manage_users(server, rooms, users_in_room));
    }

    // Spawn bot-user interaction task
    {
        let server = Arc::clone(&server);
        let rooms = Arc::clone(&rooms);
        let users_in_room = Arc::clone(&users_in_room);
        tokio::spawn(manage_bot_interactions(server, rooms, users_in_room));
    }

    // Run for specified duration then exit
    sleep(duration).await;
    println!("[LOG]: Done with random simulation. Exiting.");
}
