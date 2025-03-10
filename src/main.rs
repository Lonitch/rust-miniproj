#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
  // Wait for user input
  let stdin = io::stdin();
  let mut input = String::new();
  loop {
    print!("$ ");
    io::stdout().flush().unwrap();
    stdin.read_line(&mut input).unwrap();
    // exit cmd
    if input.contains("exit") {
      let res = input.split(' ').last().unwrap();
      match res.trim().parse::<i32>() {
        Ok(num) => std::process::exit(num),
        Err(_) => println!("Invalid exit command"),
      }
    }
    if input.contains("echo") {
      let msg = input.split("echo").last().unwrap().trim();
      println!("{}", msg);
    } else {
      println!("{}: command not found", input.trim());
    }
    input.clear();
  }
}
