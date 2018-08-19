use std::error::Error;

use ::log::LevelFilter;
use ::config_rs::ConfigError;

use util;

#[cfg(build = "debug")]
const ROCKET_ENVIRONMENT: ::rocket::config::Environment = ::rocket::config::Environment::Development;
#[cfg(build = "release")]
const ROCKET_ENVIRONMENT: ::rocket::config::Environment = ::rocket::config::Environment::Production;

#[derive(Debug, Deserialize)]
pub struct RocketConfig {
    address: String,
    port: u16,
}
impl RocketConfig {
    pub fn get(&self) -> ::rocket::config::Config {
        ::rocket::config::Config::build(ROCKET_ENVIRONMENT)
            .address(self.address.clone())
            .log_level(util::rocket_log_level(::log::max_level()))
            .port(self.port)
            .unwrap()
    }
}
#[derive(Debug, Deserialize)]
pub struct Config {
    log_level: String,
    libvirt_uri: String,
    http: RocketConfig,

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
    pub fn libvirt_uri(&self) -> &str {
        &self.libvirt_uri
    }
    pub fn http(&self) -> &RocketConfig {
        &self.http
    }
}
