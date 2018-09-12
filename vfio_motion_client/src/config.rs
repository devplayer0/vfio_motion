use std::error::Error;
use std::path::{Path, PathBuf};

use ::log::LevelFilter;
use ::config_rs::ConfigError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Libvirt {
    pub uri: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Http {
    pub url: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    log_level: String,
    pub log_dir: String,

    pub libvirt: Libvirt,
    pub http: Http,

    pub domain: String,
    pub devices: Vec<String>,

    #[serde(skip)]
    pub is_service: bool,
    #[serde(skip)]
    _log_level: Option<LevelFilter>,
}
impl Config {
    pub fn log_level(&mut self) -> Result<LevelFilter, ConfigError> {
        match self._log_level {
            Some(v) => Ok(v),
            None => {
                let v = self.log_level.parse().map_err(|e: ::log::ParseLevelError| ::config_rs::ConfigError::Message(e.description().to_string()))?;
                self._log_level = Some(v);
                Ok(v)
            }
        }
    }
    pub fn log_file(&self) -> PathBuf {
        Path::new(&self.log_dir).join(
            if self.is_service {
                "service.log"
            } else {
                "gui.log"
            }
        )
    }
}
