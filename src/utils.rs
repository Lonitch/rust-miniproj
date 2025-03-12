use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

pub fn parse_args(input: &str) -> Vec<String> {
  let mut args = Vec::new();
  let mut current_arg = String::new();
  let mut in_single_quotes = false;
  let mut in_double_quotes = false;
  let mut escaped = false;

  for c in input.chars() {
    if escaped {
      // Handle escaped character
      current_arg.push(c);
      escaped = false;
    } else if c == '\\' {
      // Escape the next character
      escaped = true;
    } else if c == '\'' && !in_double_quotes {
      // Toggle single quotes
      in_single_quotes = !in_single_quotes;
    } else if c == '"' && !in_single_quotes {
      // Toggle double quotes
      in_double_quotes = !in_double_quotes;
    } else if c.is_whitespace() && !in_single_quotes && !in_double_quotes {
      // End of argument
      if !current_arg.is_empty() {
        args.push(current_arg);
        current_arg = String::new();
      }
    } else {
      current_arg.push(c);
    }
  }

  if !current_arg.is_empty() {
    args.push(current_arg);
  }

  args
}

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

pub fn cmd_exec(cmd: &Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
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
