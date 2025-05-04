use serde::{Deserialize, Serialize};
use std::{fmt, fs, path::PathBuf};
use tracing::Level as TracingLevel;

use super::IoResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Level(pub TracingLevel);

impl Level {
    pub const TRACE: Self = Self(tracing::Level::TRACE);
    pub const DEBUG: Self = Self(tracing::Level::DEBUG);
    pub const INFO: Self = Self(tracing::Level::INFO);
    pub const WARN: Self = Self(tracing::Level::WARN);
    pub const ERROR: Self = Self(tracing::Level::ERROR);
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub tls: TlsConfig,
    pub network: NetworkConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TlsConfig {
    pub cert: Option<PathBuf>,
    pub key: Option<PathBuf>,
    pub quic: bool,
    pub enable: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NetworkConfig {
    pub bind: String, // ip:port
    pub quic_port: Option<u16>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LoggingConfig {
    pub level: String,
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::TRACE => write!(f, "TRACE"),
            Self::DEBUG => write!(f, "DEBUG"),
            Self::INFO => write!(f, "INFO"),
            Self::WARN => write!(f, "WARN"),
            Self::ERROR => write!(f, "ERROR"),
        }
    }
}

impl From<&str> for Level {
    fn from(s: &str) -> Self {
        match s {
            "TRACE" => Self::TRACE,
            "DEBUG" => Self::DEBUG,
            "INFO" => Self::INFO,
            "WARN" => Self::WARN,
            "ERROR" => Self::ERROR,
            _ => panic!("invalid level"),
        }
    }
}

impl From<String> for Level {
    fn from(s: String) -> Self {
        Self::from(s.as_str())
    }
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config = match fs::read_to_string("creme-brulee.toml") {
            Ok(config) => config,
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    let config = Config::default();
                    config.write()?;
                    return Ok(config);
                }
                return Err(e.into());
            }
        };

        let config: Config = toml::from_str(&config)?;

        Ok(config)
    }

    pub fn tls(&self) -> &TlsConfig {
        &self.tls
    }

    pub fn network(&self) -> &NetworkConfig {
        &self.network
    }

    pub fn logging(&self) -> &LoggingConfig {
        &self.logging
    }

    pub fn default() -> Self {
        Self {
            tls: TlsConfig {
                cert: None,
                key: None,
                quic: false,
                enable: false,
            },
            network: NetworkConfig {
                bind: "0.0.0.0:8080".to_string(),
                quic_port: None,
            },
            logging: LoggingConfig {
                level: "INFO".to_string(),
            },
        }
    }

    pub fn write(&self) -> IoResult<()> {
        let config = toml::to_string_pretty(self)
            .map_err(|e| panic!("Couldn't serialize config: {e}"))
            .unwrap();
        fs::write("creme-brulee.toml", config)?;

        Ok(())
    }
}
