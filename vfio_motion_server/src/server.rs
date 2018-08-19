extern crate rocket;

use self::rocket::config::Config;
use self::rocket::error::{LaunchError};

pub fn run(config: Config) -> LaunchError {
    rocket::custom(config, true).launch()
}
