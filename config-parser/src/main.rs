use config_parser::{format_config, parse_config};
use std::{env, fs};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Get the JSON file path from command line arguments.
    // 2. Read the file contents.
    // 3. Parse the JSON into Config struct using parse_config.
    // 4. Validate the Config using the validate method.
    // 5. Serialize the Config back to a formatted JSON string using format_config.
    // 6. Print the formatted JSON.

    println!("Config Parser Project");

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <config_file>", args[0]);
        std::process::exit(1);
    }
    let config_path = &args[1];
    let config_str = fs::read_to_string(config_path)?;
    let config = parse_config(&config_str)?;
    let formatted_config = format_config(&config)?;
    println!("{}", formatted_config);
    Ok(())
}
