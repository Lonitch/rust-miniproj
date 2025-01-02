use std::collections::HashMap;
use crate::chat_room::ChatRoom;
use crate::error::{ChatError, Result};
use crate::message::ChatMessage;

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

    pub async fn remove_room(&mut self, name: &str) -> Result<()> {
        let room = self.rooms.get(name).ok_or(ChatError::RoomNotFound)?;

        if !room.is_empty() {
            return Err(ChatError::RoomNotEmpty);
        }

        self.rooms.remove(name).ok_or(ChatError::RoomNotFound)?;
        Ok(())
    }

    pub async fn join_room(&mut self, room: &str, user: String) -> Result<()> {
        let chat_room = self.rooms.get_mut(room).ok_or(ChatError::RoomNotFound)?;

        chat_room.add_user(user.clone()).await;

        let mut broadcast_rx = chat_room.subscribe();

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
        chat_room.remove_user(user);
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
