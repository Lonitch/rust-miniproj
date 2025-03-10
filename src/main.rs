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
    if input.contains("exit") {
      let res = input.split(' ').last().unwrap();
      match res.trim().parse::<i32>() {
        Ok(num) => std::process::exit(num),
        Err(_) => println!("Invalid exit command"),
      }
    } else {
      println!("{}: command not found", input.trim());
    }
    input.clear();
  }
}
