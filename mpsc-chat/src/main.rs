use mpsc_chat::{ChatServer, Result, Command, parse_command, handle_command, ChatMessage};
use std::io::{self, Write};
use rand::{SeedableRng, seq::SliceRandom};
use std::sync::Arc;
use tokio::sync::Mutex;

const BOT_NAMES: [&str; 2] = ["general-bot", "random-bot"];
const INITIAL_ROOMS: [&str; 2] = ["general", "random"];
const BOT_RESPONSES: [&str; 10] = [
    "Interesting point!",
    "I see what you mean.",
    "That's fascinating.",
    "Tell me more about that.",
    "I hadn't thought of it that way.",
    "Very insightful!",
    "Good observation.",
    "That's a unique perspective.",
    "I agree completely.",
    "You make a good point."
];


#[tokio::main]
async fn main() -> Result<()> {
    let server = Arc::new(Mutex::new(ChatServer::new().await));
    
    // create initial rooms
    for room in INITIAL_ROOMS {
        server.lock().await.create_room(room.to_string()).await?;
    }
    
    // set up bots in initial rooms
    {
        let mut server = server.lock().await;
        server.join_room("general", BOT_NAMES[0].to_string()).await?;
        server.join_room("random", BOT_NAMES[1].to_string()).await?;
    }
    
    print!("Enter your username: ");
    io::stdout().flush().unwrap();
    let mut username = String::new();
    io::stdin().read_line(&mut username).unwrap();
    let username = username.trim().to_string();
    
    println!("Welcome {}! Type /help for available commands.", username);
    
    let mut current_room = None;
    let mut input = String::new();
    let mut rng = rand::rngs::StdRng::from_entropy();
    
    loop {
        input.clear();
        print!("> ");
        io::stdout().flush().unwrap();
        
        if io::stdin().read_line(&mut input).unwrap() == 0 {
            break;
        }
        
        let cmd = input.trim();
        if cmd.is_empty() {
            continue;
        }
        
        if let Some(command) = parse_command(cmd) {
            let mut server_guard = server.lock().await;
            
            // handle the command and trigger bot response if it's a message
            let result = match command {
                Command::CreateRoom(room_name) => {
                let bot_name = format!("{}-bot", room_name);
                let room_name = room_name.clone();
                
                if server_guard.create_room(room_name.clone()).await.is_ok() {
                    let _ = server_guard.join_room(&room_name, bot_name.clone()).await;
                }
                Ok(true)
                }
                Command::SendMessage(content) => {
                    let res = handle_command(Command::SendMessage(content.clone()), &mut *server_guard, &username, &mut current_room).await;
                    if res.is_ok() {
                        if let Some(room) = &current_room {
                            let bot_name = format!("{}-bot", room);
                            
                            // bot response
                            let response = BOT_RESPONSES.choose(&mut rng).unwrap();
                            let bot_message = ChatMessage {
                                sender: bot_name,
                                room: room.clone(),
                                content: response.to_string(),
                                timestamp: chrono::Utc::now(),
                            };
                            let _ = server_guard.send_message(room, bot_message).await;
                        }
                    }
                    res
                },
                cmd => handle_command(cmd, &mut *server_guard, &username, &mut current_room).await,
            };
            
            match result {
                Ok(true) => continue,
                Ok(false) => break,
                Err(e) => println!("Error: {:?}", e),
            }
        }
    }
    
    println!("Goodbye!");
    Ok(())
}
