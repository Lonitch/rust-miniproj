use super::Command;

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

pub fn handle_echo(
  cmd: &Vec<&str>,
  input: &String,
) {
  let last = input.split("echo").last().unwrap_or("").trim();
  if last.starts_with('\'') && last.ends_with('\'') {
    println!("{}", &last[1..last.len() - 1]);
  } else {
    let msg = &cmd[1..].join(" ");
    println!("{}", msg);
  }
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
