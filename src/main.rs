mod cmdline;
#[allow(unused_imports)]
mod executable;

use cmdline::Cmdline;
use executable::Executable;
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
  // Wait for user input
  let stdin = io::stdin();
  let mut input = String::new();
  loop {
    print!("$ ");
    io::stdout().flush().unwrap();
    stdin.read_line(&mut input).unwrap();
    let cmdline = Cmdline::new(&input);

    match cmdline.exec {
      Executable::Cd => executable::handle_cd(&cmdline.args),
      Executable::Exit => executable::handle_exit(&cmdline.args),
      Executable::Echo => executable::handle_echo(&cmdline.args)?,
      Executable::Pwd => executable::handle_pwd(),
      Executable::Type => executable::handle_type(&cmdline.args),
      Executable::Unknown(x) => {
        if Cmdline::executable_exists(x.as_str()) {
          Cmdline::cmd_exec(&cmdline.args)?
        } else {
          println!("{}: executable not found", input.trim());
        }
      },
    }
    input.clear();
  }
}
