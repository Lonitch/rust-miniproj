use std::collections::HashMap;
// TODO: did not use mpsc since we need to broadcast msgs to all chatroom subscriber
use tokio::sync::broadcast;

#[derive(Debug)]
pub enum ChatError
{
  RoomExists,
  RoomNotFound,
  SendError,
  ReceiveError,
}

pub type Result<T> = std::result::Result<T, ChatError>;

#[derive(Debug, Clone)]
pub struct ChatMessage
{
  pub sender: String,
  pub room: String,
  pub content: String,
  pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl std::fmt::Display for ChatMessage
{
  fn fmt(&self,
         f: &mut std::fmt::Formatter<'_>)
         -> std::fmt::Result
  {
    write!(f,
           "[{}] {} said in {}:\n {}\n",
           self.timestamp, self.sender, self.room, self.content)
  }
}

pub struct ChatRoom
{
  name: String,
  users: HashMap<String, broadcast::Sender<ChatMessage>>,
  tx: broadcast::Sender<ChatMessage>,
}

impl ChatRoom
{
  async fn new(name: String) -> ChatRoom
  {
    let (tx, _rx) = broadcast::channel(16);
    ChatRoom { name,
               users: HashMap::new(),
               tx }
  }
  async fn add_user(&mut self,
                    name: String)
  {
    self.users.insert(name, self.tx.clone());
  }
  async fn broadcast(&self,
                     message: ChatMessage)
                     -> Result<()>
  {
    self.tx
        .send(message.clone())
        .map_err(|_| ChatError::SendError)?;
    Ok(())
  }
}

pub struct ChatServer
{
  rooms: HashMap<String, ChatRoom>,
}

impl ChatServer
{
  pub async fn new() -> Self
  {
    ChatServer { rooms: HashMap::new() }
  }
  pub async fn create_room(&mut self,
                       name: String)
                       -> Result<()>
  {
    if self.rooms.contains_key(&name) {
      return Err(ChatError::RoomExists);
    }
    let chat_room = ChatRoom::new(name.clone()).await;
    self.rooms.insert(name, chat_room);
    Ok(())
  }

  // TODO: is this safe to do? any winding-down plan?
  pub async fn remove_room(&mut self,
                       name: &str)
                       -> Result<()>
  {
    if self.rooms.remove(name).is_some() {
      Ok(())
    } else {
      Err(ChatError::RoomNotFound)
    }
  }
  
  // TODO: is this a good way? joining means that you're listening to the chatroom.
  pub async fn join_room(&mut self,
                     room: &str,
                     user: String)
                     -> Result<()>
  {
    let chat_room = self.rooms
                        .get_mut(room)
                        .ok_or(ChatError::RoomNotFound)?;
    chat_room.add_user(user.clone()).await;
    chat_room.users.insert(user.clone(), chat_room.tx.clone());
    
    // only spawn print task if not a bot user
    if !user.ends_with("-bot") {
        let mut broadcast_rx = chat_room.tx.subscribe();
        tokio::spawn(async move {
            while let Ok(message) = broadcast_rx.recv().await {
                println!("{}", message);
                print!("> ");
            }
        });
    }
    Ok(())
  }
  pub async fn leave_room(&mut self,
                      room: &str,
                      user: &str)
                      -> Result<()>
  {
    let chat_room = self.rooms
                        .get_mut(room)
                        .ok_or(ChatError::RoomNotFound)?;
    chat_room.users.remove(user);
    Ok(())
  }
  pub async fn send_message(&self,
                        room: &str,
                        message: ChatMessage)
                        -> Result<()>
  {
    let chat_room = self.rooms
                        .get(room)
                        .ok_or(ChatError::RoomNotFound)?;
    chat_room.broadcast(message).await
  }
  pub fn list_rooms(&self) -> Vec<String> {
    self.rooms.keys().cloned().collect()
  }
}

pub enum Command {
    CreateRoom(String),
    JoinRoom(String),
    LeaveRoom(String),
    SendMessage(String),
    ShowCurrentRoom,
    ListRooms,
    Quit
}

pub fn parse_command(input: &str) -> Option<Command> {
    let mut parts = input.splitn(2, ' ');
    let command = parts.next()?;
    let args = parts.next().unwrap_or("").trim();

    match command {
        "/create" if !args.is_empty() => Some(Command::CreateRoom(args.to_string())),
        "/join" if !args.is_empty() => Some(Command::JoinRoom(args.to_string())),
        "/leave" => Some(Command::LeaveRoom(args.to_string())),
        "/send" if !args.is_empty() => Some(Command::SendMessage(args.to_string())),
        "/current" => Some(Command::ShowCurrentRoom),
        "/list" => Some(Command::ListRooms),
        "/quit" => Some(Command::Quit),
        _ => {
            println!("Unknown command. Available commands:");
            println!("/create <room> - Create a new chat room");
            println!("/join <room> - Join a chat room");
            println!("/leave - Leave current room");
            println!("/send <message> - Send message to current room");
            println!("/list - List available rooms");
            println!("/quit - Exit the chat");
            None
        }
    }
}

pub async fn handle_command(command: Command, server: &mut ChatServer, username: &str, current_room: &mut Option<String>) -> Result<bool> {
    match command {
        Command::CreateRoom(name) => {
            server.create_room(name.clone()).await?;
            println!("Room '{}' created successfully", name);
            Ok(true)
        }
        Command::JoinRoom(name) => {
            server.join_room(&name, username.to_string()).await?;
            *current_room = Some(name.clone());
            println!("Joined room '{}'", name);
            Ok(true)
        }
        Command::LeaveRoom(r) => {
            server.leave_room(&r, username).await?;
            println!("Left room '{}'", r);
            Ok(true)
        }
        Command::SendMessage(content) => {
            if let Some(room) = current_room.as_ref() {
                let message = ChatMessage {
                    sender: username.to_string(),
                    room: room.clone(),
                    content,
                    timestamp: chrono::Utc::now(),
                };
                server.send_message(room, message).await?;
            } else {
                println!("Not in any room. Join a room first to send messages.");
            }
            Ok(true)
        }
        Command::ShowCurrentRoom => {
            if let Some(room) = current_room.as_ref() {
              println!("You're currently in Room: {}", room);
            } else {
              println!("Not in any room.");
            }
            Ok(true)
        }
        Command::ListRooms => {
            println!("Available rooms:");
            for room in server.list_rooms() {
                println!("- {}", room);
            }
            Ok(true)
        }
        Command::Quit => Ok(false),
    }
}
