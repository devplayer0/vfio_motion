extern crate log;
extern crate rocket;

use self::rocket::config::Config;
use self::rocket::error::{LaunchError};

pub fn run(config: Config) -> LaunchError {
    // Unfortunately since were using the same log framework as Rocket, log to false has no effect
    rocket::custom(config, log::max_level() >= log::LevelFilter::Debug).launch()
}
