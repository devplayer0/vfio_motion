use std::collections::HashMap;

use ::config_rs::{Source, Value, ConfigError};
use ::log::LevelFilter;
use ::rocket::config::LoggingLevel;

#[derive(Clone, Debug)]
pub struct SingleItemSource(String, String);
impl SingleItemSource {
    pub fn new(key: &str, value: String) -> Self {
        SingleItemSource(key.to_string(), value)
    }
}
impl<'a> Source for SingleItemSource {
    fn clone_into_box(&self) -> Box<Source + Send + Sync> {
        Box::new((*self).clone())
    }
    fn collect(&self) -> Result<HashMap<String, Value>, ConfigError> {
        let mut map = HashMap::with_capacity(1);
        map.insert(self.0.clone(), Value::new(None, self.1.clone()));
        Ok(map)
    }
}

pub fn rocket_log_level(level: LevelFilter) -> LoggingLevel {
    match level {
        LevelFilter::Off | LevelFilter::Error | LevelFilter::Warn => LoggingLevel::Critical,
        LevelFilter::Info => LoggingLevel::Normal,
        LevelFilter::Debug | LevelFilter::Trace => LoggingLevel::Debug,
    }
}
