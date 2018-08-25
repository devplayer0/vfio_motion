use ::log::LevelFilter;

pub struct Config {
    log_level: LevelFilter,

    host: String,
    domain: String,
    devices: Vec<String>,
}
impl Config {
    pub fn log_level(&self) -> LevelFilter {
        self.log_level
    }
    pub fn host(&self) -> &str {
        &self.host
    }
    pub fn domain(&self) -> &str {
        &self.domain
    }
    pub fn devices(&self) -> &Vec<String> {
        &self.devices
    }
}
impl Default for Config {
    fn default() -> Config {
        Config {
            log_level: LevelFilter::Info,
            host: String::from("http://10.0.122.1:3020"),
            domain: String::from("gpu"),
            devices: vec![String::from("/dev/input/by-id/usb-Logitech_G203_Prodigy_Gaming_Mouse_0487365B3837-event-mouse"), String::from("/dev/input/by-id/usb-04d9_USB_Keyboard-event-kbd")],
        }
    }
}
