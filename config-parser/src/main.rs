use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::env;

mod lib;
use lib::{Config, parse_config, format_config, Validate};

fn main() {
    // TODO: Enhance the main function to read a JSON file, parse it, validate it, and print the result.
    // Hint:
    // 1. Get the JSON file path from command line arguments.
    // 2. Read the file contents.
    // 3. Parse the JSON into Config struct using parse_config.
    // 4. Validate the Config using the validate method.
    // 5. Serialize the Config back to a formatted JSON string using format_config.
    // 6. Print the formatted JSON.

    println!("Config Parser Project");

    // Example steps:
    // let args: Vec<String> = env::args().collect();
    // if args.len() != 2 {
    //     eprintln!("Usage: {} <config_file>", args[0]);
    //     std::process::exit(1);
    // }
    // let config_path = &args[1];
    // let config_data = fs::read_to_string(config_path)?;
    // let config = parse_config(&config_data)?;
    // config.validate()?;
    // let formatted_config = format_config(&config)?;
    // println!("{}", formatted_config);
}
