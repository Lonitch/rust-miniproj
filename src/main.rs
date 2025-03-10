#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
  let cmd_headers = ["echo", "type", "exit"];
  // Wait for user input
  let stdin = io::stdin();
  let mut input = String::new();
  loop {
    print!("$ ");
    io::stdout().flush().unwrap();
    stdin.read_line(&mut input).unwrap();
    let cmd = input.split(' ').collect::<Vec<&str>>();

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
          println!("{second}: not found");
        }
      },
      _ => {
        println!("{}: command not found", input.trim());
      },
    }
    input.clear();
  }
}
