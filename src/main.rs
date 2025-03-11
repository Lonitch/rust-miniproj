#[allow(unused_imports)]
use std::io::{self, Write};
mod utils;

enum Command {
  Pwd,
  Echo,
  Exit,
  Type,
  Unknown(String),
}

impl From<&str> for Command {
  fn from(s: &str) -> Self {
    match s {
      "echo" => Command::Echo,
      "exit" => Command::Exit,
      "pwd" => Command::Pwd,
      "type" => Command::Type,
      other => Command::Unknown(other.to_string()),
    }
  }
}

impl Command {
  fn is_builtin(s: &str) -> bool {
    match Command::from(s) {
      Command::Unknown(_) => false,
      _ => true,
    }
  }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
  // Wait for user input
  let stdin = io::stdin();
  let mut input = String::new();
  loop {
    print!("$ ");
    io::stdout().flush().unwrap();
    stdin.read_line(&mut input).unwrap();
    let cmd = input.split_whitespace().collect::<Vec<&str>>();
    let exec = Command::from(*cmd.first().unwrap());

    match exec {
      Command::Exit => {
        let res = cmd.last().unwrap();
        match res.trim().parse::<i32>() {
          Ok(num) => std::process::exit(num),
          Err(_) => println!("Invalid exit command"),
        }
      },

      Command::Echo => {
        let msg = input.split("echo").last().unwrap().trim();
        println!("{}", msg);
      },

      Command::Pwd => match std::env::current_dir() {
        Ok(path) => println!("{}", path.display()),
        _ => {},
      },

      Command::Type => {
        let second = cmd.last().unwrap().trim();
        if Command::is_builtin(second) {
          println!("{second} is a shell builtin");
        } else {
          if let Ok(path) = std::env::var("PATH") {
            let mut cmd_exist = false;
            for dir in path.split(':') {
              let p = std::path::Path::new(dir).join(second);
              if p.exists() {
                println!("{second} is {}", p.to_str().unwrap());
                cmd_exist = true;
                break;
              }
            }
            if !cmd_exist {
              println!("{second}: not found");
            }
          } else {
            println!("{second}: not found");
          }
        }
      },
      Command::Unknown(x) => {
        if utils::command_exists(x.as_str()) {
          utils::cmd_exec(cmd)?
        } else {
          println!("{}: command not found", input.trim());
        }
      },
    }
    input.clear();
  }
}
