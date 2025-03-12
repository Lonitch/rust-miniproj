use super::Command;
use crate::utils::parse_args;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;

pub fn handle_cd(cmd: &Vec<String>) {
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

pub fn handle_exit(cmd: &Vec<String>) {
  let res = cmd.last().unwrap();
  match res.trim().parse::<i32>() {
    Ok(num) => std::process::exit(num),
    Err(_) => println!("Invalid exit command"),
  }
}

pub fn handle_echo(
  args: &Vec<String>,
  input: &str,
) -> Result<(), std::io::Error> {
  // Check for -n flag
  let mut no_newline = false;
  let mut arg_index = 1;

  if args.len() > 0 && args[1] == "-n" {
    no_newline = true;
    arg_index = 2;
  }

  // Build the message by joining the arguments
  let message = args[arg_index..].join(" ");

  // Check for redirection
  let redirection_index = input.find('>');
  if let Some(idx) = redirection_index {
    // Extract the file path from after the redirection
    let file_path_part = &input[idx + 1..].trim();

    // Use parse_args on just the file path part to handle quotes properly
    let file_args = parse_args(file_path_part);
    if file_args.is_empty() {
      return Err(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        "Missing file path after redirection",
      ));
    }

    let file_path = &file_args[0];

    // Ensure parent directory exists
    if let Some(parent) = std::path::Path::new(&file_path).parent() {
      if !parent.exists() {
        std::fs::create_dir_all(parent)?;
      }
    }

    // Open file and write content
    let mut file = std::fs::OpenOptions::new()
      .write(true)
      .create(true)
      .truncate(true)
      .open(file_path)?;

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

pub fn handle_pwd() {
  match std::env::current_dir() {
    Ok(path) => println!("{}", path.display()),
    _ => {},
  }
}

pub fn handle_type(cmd: &Vec<String>) {
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
