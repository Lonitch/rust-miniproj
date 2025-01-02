use std::collections::HashMap;
// TODO: did not use mpsc since we need to broadcast msgs to all chatroom subscriber
use tokio::sync::broadcast;

#[derive(Debug)]
pub enum ChatError {
    RoomExists,
    RoomNotFound,
    SendError,
    ReceiveError,
    RoomNotEmpty,
}

pub type Result<T> = std::result::Result<T, ChatError>;

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub sender: String,
    pub room: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl std::fmt::Display for ChatMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {} said in {}:\n {}\n",
            self.timestamp, self.sender, self.room, self.content
        )
    }
}

pub struct ChatRoom {
    users: HashMap<String, broadcast::Sender<ChatMessage>>,
    tx: broadcast::Sender<ChatMessage>,
}

impl ChatRoom {
    fn is_empty(&self) -> bool {
        self.users.is_empty()
    }

    async fn new() -> ChatRoom {
        let (tx, _rx) = broadcast::channel(16);
        ChatRoom {
            users: HashMap::new(),
            tx,
        }
    }
    async fn add_user(&mut self, name: String) {
        self.users.insert(name, self.tx.clone());
    }
    async fn broadcast(&self, message: ChatMessage) -> Result<()> {
        self.tx
            .send(message.clone())
            .map_err(|_| ChatError::SendError)?;
        Ok(())
    }
}

pub struct ChatServer {
    rooms: HashMap<String, ChatRoom>,
}

impl ChatServer {
    pub async fn new() -> Self {
        ChatServer {
            rooms: HashMap::new(),
        }
    }

    pub async fn create_room(&mut self, name: String) -> Result<()> {
        if self.rooms.contains_key(&name) {
            return Err(ChatError::RoomExists);
        }
        let chat_room = ChatRoom::new().await;
        self.rooms.insert(name, chat_room);
        Ok(())
    }

    // TODO: is this safe to do? any winding-down plan?
    pub async fn remove_room(&mut self, name: &str) -> Result<()> {
        let room = self.rooms.get(name).ok_or(ChatError::RoomNotFound)?;

        if !room.is_empty() {
            return Err(ChatError::RoomNotEmpty);
        }

        self.rooms.remove(name).ok_or(ChatError::RoomNotFound)?;
        Ok(())
    }

    // TODO: is this a good way? joining means that you're listening to the chatroom.
    // I suspect this function is the source cause of duplicated message printing!
    pub async fn join_room(&mut self, room: &str, user: String) -> Result<()> {
        let chat_room = self.rooms.get_mut(room).ok_or(ChatError::RoomNotFound)?;

        // Already in the room? Return early
        if chat_room.users.contains_key(&user) {
            return Ok(());
        }

        chat_room.add_user(user.clone()).await;

        let mut broadcast_rx = chat_room.tx.subscribe();

        tokio::spawn(async move {
            while let Ok(message) = broadcast_rx.recv().await {
                println!("{}", message);
                if message.sender == user {
                    continue;
                }
            }
        });
        Ok(())
    }

    pub async fn leave_room(&mut self, room: &str, user: &str) -> Result<()> {
        let chat_room = self.rooms.get_mut(room).ok_or(ChatError::RoomNotFound)?;
        chat_room.users.remove(user);
        Ok(())
    }

    pub async fn send_message(&self, room: &str, message: ChatMessage) -> Result<()> {
        let chat_room = self.rooms.get(room).ok_or(ChatError::RoomNotFound)?;
        chat_room.broadcast(message).await
    }

    pub fn list_rooms(&self) -> Vec<String> {
        self.rooms.keys().cloned().collect()
    }
}
