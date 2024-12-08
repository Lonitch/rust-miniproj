use serde::{Deserialize, Serialize};
use serde_json::from_str as from_json_str;
use std::error::Error;
use std::fmt;

// TODO: Implement custom deserialization for Mode to support case-insensitive parsing
// Hint: Implement Deserialize manually instead of deriving it
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Mode
{
  Development,
  Production,
  Testing,
}

// Optional settings
#[derive(Debug, Serialize, Deserialize)]
pub struct Settings
{
  pub theme: Option<String>,
  pub max_connections: Option<u32>,
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

// Possible config errors
#[derive(Debug)]
pub enum ConfigError
{
  EmptyName,
  EmptyVersion,
  InvalidVersion,
  EmptyFeatures,
  InvalidSettings(String),
  InvalidMode(String),
}

impl Error for ConfigError {}

impl fmt::Display for ConfigError
{
  fn fmt(&self,
         f: &mut fmt::Formatter<'_>)
         -> fmt::Result
  {
    match self {
      ConfigError::EmptyName => write!(f, "Name CANNOT be empty"),
      ConfigError::EmptyVersion => write!(f, "Version CANNOT be empty"),
      ConfigError::InvalidVersion => write!(f, "Version must follow X.Y.Z format"),
      ConfigError::EmptyFeatures => write!(f, "No Feature is found"),
      ConfigError::InvalidSettings(msg) => write!(f, "Invalid settings: {}", msg),
      ConfigError::InvalidMode => write!(f,
                                         "Invalid mode: must be Devlopment, Testing, or Production"),
      _ => write!(f, "Unknown Error"),
    }
  }
}

pub trait Validate
{
  fn validate(&mut self) -> Result<(), Box<dyn Error>>;
}

impl Validate for Config
{
  fn validate(&mut self) -> Result<(), ConfigError>
  {
    // - Check if name is non-empty
    // - Check if version is non-empty and follows semver format
    // - Validate settings and assign default values
    // - Ensure features array is not empty
    // - Check if value of Mode is one of the enum
    if self.name.len() == 0 {
      Err(ConfigError::EmptyName)
    }

    if self.version.len() == 0 {
      Err(ConfigError::EmptyVersion)
    }

    if !self.version
            .split('.')
            .filter(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()))
            .count()
       == 3
    {
      return Err(ConfigError::InvalidVersion);
    }

    if !self.features.len() == 0 {
      return Err(ConfigError::EmptyFeatures);
    }

    // TODO: is this the right way to set defaults? should I impl Default?
    if self.settings.theme.is_none() {
      self.settings.theme = Some("default".to_string());
    }
    if self.settings.max_connections.is_none() {
      self.settings.max_connections = Some(10u32);
    }

    Ok(())
  }
}

pub fn parse_config(json: &str) -> Result<Config, ConfigError>
{
  // TODO: format is weird, don't know how to shift the "if" forward in rustfmt.toml
  let mut config: Config = from_json_str(json).map_err(|e| {
                                                // TODO: a dirt workaround, what is the better way?
                                                if e.to_string().contains("mode") {
                                                  ConfigError::InvalidMode(e.to_string())
                                                } else {
                                                  ConfigError::InvalidSettings(e.to_string())
                                                }
                                              })?;

  config.validate()?;

  Ok(config)
}

pub fn format_config(config: &Config) -> Result<String, Box<dyn Error>>
{
  let json = serde_json::to_string_pretty(config)?;
  Ok(json)
}
