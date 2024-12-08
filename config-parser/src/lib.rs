use serde::{Deserialize, Serialize};
use std::error::Error;

// TODO: Implement custom deserialization for Mode to support case-insensitive parsing
// Hint: Implement Deserialize manually instead of deriving it
#[derive(Debug, Serialize, Deserialize, PartialEq)]
// TODO: Derive PartialEq for the Mode enum to allow comparison in tests
pub enum Mode
{
  Development,
  Production,
  Testing,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings
{
  pub theme: Option<String>,
  pub max_connections: Option<u32>,
}

impl Default for Settings
{
  fn default() -> Self
  {
    Self { theme: Some("default".to_string()),
           max_connections: Some(10) }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config
{
  pub name: String,
  pub version: String,
  pub settings: Settings,
  pub features: Vec<String>,
  pub mode: Mode,
}

// TODO: Implement this trait to validate configuration fields
pub trait Validate
{
  fn validate(&self) -> Result<(), Box<dyn Error>>;
}

impl Validate for Config
{
  fn validate(&self) -> Result<(), Box<dyn Error>>
  {
    // TODO: Implement validation logic
    // - Check if name is non-empty
    // - Check if version is non-empty and follows semver format
    // - Validate settings (if any custom validation is needed)
    // - Ensure features array is not empty
    unimplemented!("Implement validation logic for Config")
  }
}

// TODO: Implement this function to parse and validate config from JSON
pub fn parse_config(json: &str) -> Result<Config, Box<dyn Error>>
{
  unimplemented!("Implement config parsing from JSON string")
}

// TODO: Implement this function to format and serialize config to JSON
pub fn format_config(config: &Config) -> Result<String, Box<dyn Error>>
{
  unimplemented!("Implement config formatting to JSON string")
}
