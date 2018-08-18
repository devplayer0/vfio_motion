use std::collections::HashMap;

extern crate config;

use config::{Source, Value, ConfigError};

#[derive(Clone, Debug)]
pub struct SingleItemSource(String, String);
impl SingleItemSource {
    pub fn from(key: &str, value: String) -> Self {
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
