use crate::Executable;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

pub struct Cmdline {
  // The command to execute (corresponds to existing Command enum)
  pub exec: Executable,

  // Arguments for the command
  pub args: Vec<String>,

  // Redirection information
  pub stdout_redirect: Option<RedirectInfo>,
  pub stderr_redirect: Option<RedirectInfo>,
  pub stdin_redirect: Option<RedirectInfo>,
}

pub struct RedirectInfo {
  pub path: String,
  // descriptor number (default: 1 for stdout)
  pub fd: u32,
  // redirection mode
  pub mode: RedirectMode,
}

pub enum RedirectMode {
  // > - Write to file (create or truncate)
  Write,
  // >| - Force write even with noclobber
  ForceWrite,
  // >> - Append to file
  Append,
  // < - Read from file
  Read,
}

impl Cmdline {
  pub fn new(input: &str) -> Self {
    let args = Cmdline::parse_args(input);
    let exec = if let Some(cmd_name) = args.first() {
      Executable::from(cmd_name.clone())
    } else {
      Executable::Unknown("".to_string())
    };
    Self {
      exec,
      args,
      stdout_redirect: None,
      stderr_redirect: None,
      stdin_redirect: None,
    }
  }

  pub fn parse_args(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current_arg = String::new();
    let mut in_single_quotes = false;
    let mut in_double_quotes = false;
    let mut escaped = false;

    for c in input.chars() {
      if escaped {
        if in_double_quotes {
          if c == '$' || c == '`' || c == '"' || c == '\\' || c == '\n' {
            current_arg.push(c);
          } else {
            current_arg.push('\\');
            current_arg.push(c);
          }
        } else if in_single_quotes {
          // Inside single quotes, backslash has no special meaning
          current_arg.push('\\');
          current_arg.push(c);
        } else if c == '\n' {
          // Line continuation outside quotes
          // Skip both the backslash and newline
        } else {
          // Outside quotes, backslash escapes any character
          current_arg.push(c);
        }
        escaped = false;
      } else if c == '\\' {
        // Potential escape sequence
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

    // Handle any trailing escape character
    if escaped {
      if !in_single_quotes && !in_double_quotes {
        current_arg.push('\\');
      } else if in_single_quotes {
        current_arg.push('\\');
      }
      // In double quotes, a trailing backslash is just a backslash
      if in_double_quotes {
        current_arg.push('\\');
      }
    }

    // Add the last argument if not empty
    if !current_arg.is_empty() {
      args.push(current_arg);
    }

    args
  }

  pub fn executable_exists(command: &str) -> bool {
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
}
