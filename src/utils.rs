use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

pub fn command_exists(command: &str) -> bool {
  if let Ok(path) = std::env::var("PATH") {
    for dir in path.split(':') {
      let path = std::path::Path::new(dir).join(command.trim());
      if path.exists() {
        return true;
      }
    }
  }
  false
}

pub fn cmd_exec(cmd: Vec<&str>) -> Result<(), Box<dyn std::error::Error>> {
  if let Some((prog, args)) = cmd.split_first() {
    let mut child = Command::new(prog)
      .args(args)
      .stdout(Stdio::piped())
      .spawn()?;

    if let Some(stdout) = child.stdout.take() {
      let reader = BufReader::new(stdout);
      for line in reader.lines() {
        println!("{}", line?);
      }
    }

    child.wait()?;
  }
  Ok(())
}
