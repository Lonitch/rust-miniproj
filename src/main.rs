#[allow(unused_imports)]
mod command;
mod utils;

use command::Command;
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
  // Wait for user input
  let stdin = io::stdin();
  let mut input = String::new();
  loop {
    print!("$ ");
    io::stdout().flush().unwrap();
    stdin.read_line(&mut input).unwrap();
    let cmd = utils::parse_args(&input);
    let exec = if let Some(cmd_name) = cmd.first() {
      Command::from(cmd_name.clone())
    } else {
      Command::Unknown("".to_string())
    };

    match exec {
      Command::Cd => command::handle_cd(&cmd),
      Command::Exit => command::handle_exit(&cmd),
      Command::Echo => command::handle_echo(&cmd, &input)?,
      Command::Pwd => command::handle_pwd(),
      Command::Type => command::handle_type(&cmd),
      Command::Unknown(x) => {
        if utils::command_exists(x.as_str()) {
          utils::cmd_exec(&cmd)?
        } else {
          println!("{}: command not found", input.trim());
        }
      },
    }
    input.clear();
  }
}
