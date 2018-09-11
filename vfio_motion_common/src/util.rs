use std::collections::HashMap;
use ::config::{Source, Value, ConfigError};

#[macro_export]
macro_rules! merge_arg {
    ($args:ident, $config:ident, $key:expr) => (
        if $args.is_present($key) {
            $config.merge(SingleItemSource::new($key, $args.value_of($key).unwrap().to_string()))?;
        }
    )
}

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
