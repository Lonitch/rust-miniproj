use std::collections::HashMap;
// TODO: did not use mpsc since we need to broadcast msgs to all chatroom subscriber
use crate::error::{ChatError, ChatResult};
use crate::message::ChatMessage;
use tokio::sync::broadcast;

pub struct ChatRoom {
    users: HashMap<String, broadcast::Sender<ChatMessage>>,
    tx: broadcast::Sender<ChatMessage>,
}

impl ChatRoom {
    pub fn is_empty(&self) -> bool {
        self.users.is_empty()
    }

    pub async fn new() -> ChatRoom {
        let (tx, _rx) = broadcast::channel(16);
        ChatRoom {
            users: HashMap::new(),
            tx,
        }
    }

    pub async fn add_user(&mut self, name: String) {
        self.users.insert(name, self.tx.clone());
    }

    pub async fn broadcast(&self, message: ChatMessage) -> ChatResult<()> {
        self.tx
            .send(message.clone())
            .map_err(|_| ChatError::SendError)?;
        Ok(())
    }

    pub fn remove_user(&mut self, name: &str) {
        self.users.remove(name);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ChatMessage> {
        self.tx.subscribe()
    }
}
