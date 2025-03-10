#[allow(unused_imports)]
use std::io::{self, Write};
mod utils;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let cmd_headers = ["echo", "type", "exit"];
  // Wait for user input
  let stdin = io::stdin();
  let mut input = String::new();
  loop {
    print!("$ ");
    io::stdout().flush().unwrap();
    stdin.read_line(&mut input).unwrap();
    let cmd = input.split_whitespace().collect::<Vec<&str>>();

    match *cmd.first().unwrap() {
      "exit" => {
        let res = cmd.last().unwrap();
        match res.trim().parse::<i32>() {
          Ok(num) => std::process::exit(num),
          Err(_) => println!("Invalid exit command"),
        }
      },

      "echo" => {
        let msg = input.split("echo").last().unwrap().trim();
        println!("{}", msg);
      },

      "type" => {
        let second = cmd.last().unwrap().trim();
        if cmd_headers.contains(&second) {
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
      },
      x => {
        if utils::command_exists(x) {
          utils::cmd_exec(cmd)?
        } else {
          println!("{}: command not found", input.trim());
        }
      },
    }
    input.clear();
  }
}
