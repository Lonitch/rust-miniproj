mod builtin_handlers;

pub use builtin_handlers::*;

pub enum Command {
  Cd,
  Echo,
  Exit,
  Pwd,
  Type,
  Unknown(String),
}

impl From<&str> for Command {
  fn from(s: &str) -> Self {
    match s {
      "cd" => Command::Cd,
      "echo" => Command::Echo,
      "exit" => Command::Exit,
      "pwd" => Command::Pwd,
      "type" => Command::Type,
      other => Command::Unknown(other.to_string()),
    }
  }
}

impl Command {
  pub fn is_builtin(s: &str) -> bool {
    match Command::from(s) {
      Command::Unknown(_) => false,
      _ => true,
    }
  }
}
