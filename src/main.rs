mod cmdline;
#[allow(unused_imports)]
use cmdline::Cmd;
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Wait for user input
    let stdin = io::stdin();
    let mut input = String::new();
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        stdin.read_line(&mut input).unwrap();
        let cmdline = Cmd::new(&input);
        cmdline.handle_execs()?;
        input.clear();
    }
}
