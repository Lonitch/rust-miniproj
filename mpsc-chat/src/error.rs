#[derive(Debug)]
pub enum ChatError {
    RoomExists,
    RoomNotFound,
    SendError,
    ReceiveError,
    RoomNotEmpty,
}

pub type Result<T> = std::result::Result<T, ChatError>;
