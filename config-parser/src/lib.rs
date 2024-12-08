use serde::{Deserialize, Serialize};
use serde_json::from_str as from_json_str;
use std::error::Error;
use std::fmt;

#[derive(Debug, Serialize, PartialEq)]
pub enum Mode
{
  Development,
  Production,
  Testing,
}

// TODO: still don't understand why lifetime is needed here
impl<'de> Deserialize<'de> for Mode
{
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de>
  {
    let s = String::deserialize(deserializer)?;
    match s.to_lowercase().as_str() {
      "development" => Ok(Mode::Development),
      "production" => Ok(Mode::Production),
      "testing" => Ok(Mode::Testing),
      _ => Err(serde::de::Error::custom("invalid mode")),
    }
  }
}

// Optional settings
#[derive(Debug, Serialize, Deserialize)]
pub struct Settings
{
  pub theme: Option<String>,
  pub max_connections: Option<u32>,
}

impl Default for Settings
{
  fn default() -> Settings
  {
    Settings { theme: Some("default".to_string()),
               max_connections: Some(10u32) }
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

// Possible config errors
#[derive(Debug)]
pub enum ConfigError
{
  EmptyName,
  EmptyVersion,
  InvalidVersion,
  EmptyFeatures,
  InvalidSettings(String),
  InvalidMode,
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
    }
  }
}

pub trait Validate
{
  fn validate(&mut self) -> Result<(), ConfigError>;
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
      return Err(ConfigError::EmptyName);
    }

    if self.version.len() == 0 {
      return Err(ConfigError::EmptyVersion);
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

    // TODO: is this the right way to set defaults?
    let defaults = Settings::default();
    if self.settings.theme.is_none() {
      self.settings.theme = defaults.theme.clone();
    }
    if self.settings.max_connections.is_none() {
      self.settings.max_connections = defaults.max_connections.clone();
    }

    Ok(())
  }
}

pub fn parse_config(json: &str) -> Result<Config, ConfigError>
{
  // TODO: format is weird, don't know how to shift the "if" forward in rustfmt.toml
  let mut config: Config = from_json_str(json).map_err(|e| {
                                                // TODO: a dirt workaround using custom err msg,
                                                // what is the better way?
                                                if e.to_string().contains("mode") {
                                                  ConfigError::InvalidMode
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
