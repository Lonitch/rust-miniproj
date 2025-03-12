use super::Executable;
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

pub fn handle_echo(args: &Vec<String>) -> Result<(), std::io::Error> {
  // Check for -n flag
  let mut no_newline = false;
  let mut arg_index = 1;
  let mut output_message = String::new();
  let mut output_target = None;

  if args.len() > 1 && args[1] == "-n" {
    no_newline = true;
    arg_index = 2;
  }

  // Process arguments to find redirection
  let mut i = arg_index;
  while i < args.len() {
    if args[i] == ">" && i + 1 < args.len() {
      // Found redirection
      output_target = Some(args[i + 1].clone());
      break;
    }

    // Add to message with space if not first argument
    if i > arg_index {
      output_message.push(' ');
    }
    output_message.push_str(&args[i]);

    i += 1;
  }

  // Handle output based on redirection
  if let Some(file_path) = output_target {
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
      write!(file, "{}", output_message)?;
    } else {
      writeln!(file, "{}", output_message)?;
    }
  } else {
    // No redirection, print to stdout
    if no_newline {
      print!("{}", output_message);
      std::io::stdout().flush()?;
    } else {
      println!("{}", output_message);
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
  if Executable::is_builtin(second) {
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
