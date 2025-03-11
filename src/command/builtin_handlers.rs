use super::Command;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;

pub fn handle_cd(cmd: &Vec<&str>) {
  let dir = cmd.get(1).map_or("", |s| s.trim());
  let path = if dir.starts_with("~") {
    if let Ok(home) = std::env::var("HOME") {
      if dir == "~" {
        std::path::PathBuf::from(home)
      } else {
        std::path::Path::new(&home).join(&dir[1..])
      }
    } else {
      std::path::PathBuf::from(dir)
    }
  } else {
    std::path::PathBuf::from(dir)
  };

  if let Err(_) = std::env::set_current_dir(&path) {
    println!(
      "cd: {}: No such file or directory",
      path.display()
    );
  }
}

pub fn handle_exit(cmd: &Vec<&str>) {
  let res = cmd.last().unwrap();
  match res.trim().parse::<i32>() {
    Ok(num) => std::process::exit(num),
    Err(_) => println!("Invalid exit command"),
  }
}

pub fn handle_echo(input: &str) -> Result<(), std::io::Error> {
  let input = input.trim_start_matches("echo").trim();

  // Parse the -n flag
  let (no_newline, content) = if input.starts_with("-n") {
    (true, input[2..].trim())
  } else {
    (false, input)
  };

  // Find redirection operator outside quotes
  let mut parts = split_on_unquoted(content, '>');
  let message = parts.remove(0).trim();
  let file_path = parts.first().map(|s| s.trim());

  // Parse the message (handle quotes)
  let message = parse_quoted_string(message);

  // Handle redirection
  if let Some(file_path) = file_path {
    let file_path = parse_quoted_string(file_path);

    // Ensure parent directory exists
    if let Some(parent) = Path::new(&file_path).parent() {
      std::fs::create_dir_all(parent)?;
    }

    // Open file and write content
    let mut file = OpenOptions::new()
      .write(true)
      .create(true)
      .truncate(true)
      .open(&file_path)?;

    if no_newline {
      write!(file, "{}", message)?;
    } else {
      writeln!(file, "{}", message)?;
    }
  } else {
    // No redirection, print to stdout
    if no_newline {
      print!("{}", message);
      std::io::stdout().flush()?;
    } else {
      println!("{}", message);
    }
  }

  Ok(())
}

// Helper function to parse quoted strings
fn parse_quoted_string(input: &str) -> String {
  let input = input.trim();

  if (input.starts_with('"') && input.ends_with('"')) || (input.starts_with('\'') && input.ends_with('\'')) {
    // Remove the quotes
    input[1..input.len() - 1].to_string()
  } else {
    input.to_string()
  }
}

// Helper function to split on unquoted delimiters
fn split_on_unquoted(
  s: &str,
  delimiter: char,
) -> Vec<&str> {
  let mut result = Vec::new();
  let mut start = 0;
  let mut in_single_quotes = false;
  let mut in_double_quotes = false;

  for (i, c) in s.char_indices() {
    match c {
      '\'' if !in_double_quotes => in_single_quotes = !in_single_quotes,
      '"' if !in_single_quotes => in_double_quotes = !in_double_quotes,
      c if c == delimiter && !in_single_quotes && !in_double_quotes => {
        result.push(&s[start..i]);
        start = i + 1;
      },
      _ => {},
    }
  }

  result.push(&s[start..]);
  result
}

pub fn handle_pwd() {
  match std::env::current_dir() {
    Ok(path) => println!("{}", path.display()),
    _ => {},
  }
}

pub fn handle_type(cmd: &Vec<&str>) {
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
}
