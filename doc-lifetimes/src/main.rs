use doc_lifetimes::{handle_command, parse_markdown};
use std::io::{self, Write};
use std::{env, fs};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!(
            "Usage: {} <filename>",
            args.first().unwrap_or(&"main".to_string())
        );
        return Ok(());
    }

    let filename = &args[1];
    let content = fs::read_to_string(filename)?;
    let mut doc = parse_markdown(&content);

    // command loop
    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        let line = line.trim();
        handle_command(line, &mut doc)?;
    }
}
