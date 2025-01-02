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
