pub enum Executable {
  Cd,
  Echo,
  Exit,
  Pwd,
  Type,
  Unknown(String),
}

impl From<String> for Executable {
  fn from(s: String) -> Self {
    match s.as_str() {
      "cd" => Executable::Cd,
      "echo" => Executable::Echo,
      "exit" => Executable::Exit,
      "pwd" => Executable::Pwd,
      "type" => Executable::Type,
      other => Executable::Unknown(other.to_string()),
    }
  }
}

impl Executable {
  pub fn is_builtin(s: &str) -> bool {
    match Executable::from(s.to_string()) {
      Executable::Unknown(_) => false,
      _ => true,
    }
  }
}
