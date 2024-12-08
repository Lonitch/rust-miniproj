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
  pub name: Option<String>,
  pub version: Option<String>,
  pub settings: Option<Settings>,
  pub features: Option<Vec<String>>,
  pub mode: Option<Mode>,
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
    match &self.name {
      None => return Err(ConfigError::EmptyName),
      Some(name) if name.trim().is_empty() => return Err(ConfigError::EmptyName),
      _ => {},
    }

    // Validate version is present and follows semver format
    match &self.version {
      None => return Err(ConfigError::EmptyVersion),
      Some(version) if version.trim().is_empty() => return Err(ConfigError::EmptyVersion),
      Some(version) => {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 3
           || !parts.iter()
                    .all(|s| s.chars().all(|c| c.is_ascii_digit()))
        {
          return Err(ConfigError::InvalidVersion);
        }
      },
    }

    match &self.features {
      None => return Err(ConfigError::EmptyFeatures),
      Some(features) if features.is_empty() => return Err(ConfigError::EmptyFeatures),
      _ => {},
    }

    let mut defaults = Settings::default();
    if let Some(s) = &self.settings {
      if s.theme.is_some() {
        defaults.theme = s.theme.clone();
      }
      if s.max_connections.is_some() {
        defaults.max_connections = s.max_connections;
      }
    }
    self.settings = Some(defaults);

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
