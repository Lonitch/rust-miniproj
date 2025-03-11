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
  // Trim "echo" from the beginning
  let input = input.trim_start_matches("echo").trim();

  // Parse the -n flag
  let (no_newline, content) = if input.starts_with("-n") {
    (true, input[2..].trim())
  } else {
    (false, input)
  };

  // Check if there's a redirection
  let parts = split_on_unquoted(content, '>');
  let message_part = parts[0].trim();
  let file_path = if parts.len() > 1 { parts[1].trim() } else { "" };

  // Process the message string by removing quotes
  let message = process_quoted_string(message_part);

  // Check if we need to redirect output
  if !file_path.is_empty() {
    let clean_path = process_quoted_string(file_path);

    // Ensure parent directory exists
    if let Some(parent) = Path::new(&clean_path).parent() {
      if !parent.exists() {
        std::fs::create_dir_all(parent)?;
      }
    }

    // Open file and write content
    let mut file = OpenOptions::new()
      .write(true)
      .create(true)
      .truncate(true)
      .open(&clean_path)?;

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

// Helper function to process quoted strings - removes all quotes
fn process_quoted_string(s: &str) -> String {
  // This implementation handles both quoted and unquoted strings
  let s = s.trim();

  let mut result = String::new();
  let mut in_quotes = false;
  let mut quote_char = '\0';

  for c in s.chars() {
    match c {
      '\'' | '"' => {
        // Toggle quote state but don't add quote to output
        if !in_quotes {
          in_quotes = true;
          quote_char = c;
        } else if c == quote_char {
          in_quotes = false;
        } else {
          // This is a different quote character than what started the quoted section
          result.push(c);
        }
      },
      _ => result.push(c),
    }
  }

  result
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
        break; // Only split on the first unquoted delimiter
      },
      _ => {},
    }
  }

  // Add the rest of the string (everything after the delimiter)
  if start < s.len() {
    result.push(&s[start..]);
  } else if result.is_empty() {
    // No delimiter found, add the whole string
    result.push(s);
  }

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
