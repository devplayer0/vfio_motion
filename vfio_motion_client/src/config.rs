use std::error::Error;
use std::path::{Path, PathBuf};

use ::gdk;
use ::gtk;

use ::log::LevelFilter;
use ::config_rs::ConfigError;
use ::gui;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Logging {
    level: String,
    pub dir: String,

    #[serde(skip)]
    _level: Option<LevelFilter>,
}
impl Logging {
    pub fn level(&mut self) -> Result<LevelFilter, ConfigError> {
        match self._level {
            Some(v) => Ok(v),
            None => {
                let v = self.level.parse().map_err(|e: ::log::ParseLevelError| ::config_rs::ConfigError::Message(e.description().to_string()))?;
                self._level = Some(v);
                Ok(v)
            }
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Libvirt {
    pub uri: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Http {
    pub url: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip)]
    pub file: String,

    pub native: bool,

    pub domain: String,
    pub devices: Vec<String>,

    pub service_startup: bool,
    pub hotkey: String,

    #[serde(skip)]
    pub is_service: bool,

    // tables must be last when writing toml
    pub logging: Logging,
    pub libvirt: Libvirt,
    pub http: Http,
}
impl Config {
    pub fn file(&self) -> &str {
        &self.file
    }
    pub fn log_file(&self) -> PathBuf {
        Path::new(&self.logging.dir).join(
            if self.is_service {
                "service.log"
            } else {
                "gui.log"
            }
        )
    }
    pub fn win_hotkey(&self) -> Result<(isize, u32), ConfigError> {
        let (key, mods) = gtk::accelerator_parse(&self.hotkey);
        debug!("parsing key {:#x} with mods {:?}", key, mods);
        if mods == gdk::ModifierType::empty() && key == 0 {
            return Err(ConfigError::Message(format!("failed to parse hotkey '{}'", self.hotkey)));
        }

        gui::win_hotkey(key, mods).map_err(|e| ConfigError::Message(e.to_owned()))
    }
}
