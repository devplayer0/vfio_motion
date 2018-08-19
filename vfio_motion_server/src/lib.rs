use std::error::Error;
use std::path::Path;

#[macro_use]
extern crate quick_error;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate log;
extern crate config;
extern crate virt;

use log::LevelFilter;
use config::ConfigError;

pub mod util;
mod libvirt;
mod input;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    log_level: String,
    libvirt_uri: String,

    #[serde(skip)]
    _log_level: Option<LevelFilter>,
}
impl Config {
    pub fn log_level(&mut self) -> Result<LevelFilter, ConfigError> {
        match self._log_level {
            Some(v) => Ok(v),
            None => {
                let v = self.log_level.parse().map_err(|e: log::ParseLevelError| config::ConfigError::Message(e.description().to_string()))?;
                self._log_level = Some(v);
                Ok(v)
            }
        }
    }
    pub fn libvirt_uri(&self) -> &str {
        &self.libvirt_uri
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let conn = libvirt::Connection::open(config.libvirt_uri.as_str())?;
    debug!("Opened connection to libvirt on '{}'", conn.get_uri()?);

    let domains = conn.list_all_domains(virt::connect::VIR_CONNECT_LIST_DOMAINS_ACTIVE)?;
    for domain in domains.iter().map(|d| libvirt::Domain::from(d)) {
        info!("libvirt domain: {}", domain.get_name()?);
        input::Device::new(&domain, Path::new("/dev/input/by-id/usb-Logitech_G203_Prodigy_Gaming_Mouse_0487365B3837-event-mouse"), 0x10)?.attach()?;
        info!("attached device!");
    }

    Ok(())
}
