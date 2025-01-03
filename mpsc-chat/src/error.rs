#[derive(Debug)]
pub enum ChatError {
    RoomExists,
    RoomNotFound,
    SendError,
    ReceiveError,
    RoomNotEmpty,
}

pub type ChatResult<T> = std::result::Result<T, ChatError>;
