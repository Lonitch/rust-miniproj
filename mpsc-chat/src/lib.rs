mod chat_room;
mod chat_server;
mod error;
mod message;
mod simulation;

// Re-export the public interface
pub use chat_server::ChatServer;
pub use error::{ChatError, Result};
pub use message::ChatMessage;
pub use simulation::run_simulation;
