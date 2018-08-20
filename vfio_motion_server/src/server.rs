use ::rocket::config::Config;
use ::rocket::error::{LaunchError};
use ::rocket::request::{Request, FromRequest};
use ::rocket::response::status;
use ::rocket_contrib::Json;

use input::Device;

#[post("/", data="<device>")]
fn attach(device: Json<Device>) -> status::NoContent {
    info!("we have an evdev: {:?}", device.evdev());
    status::NoContent
}

pub fn run(config: Config) -> LaunchError {
    // Unfortunately since were using the same log framework as Rocket, log to false has no effect
    ::rocket::custom(config, ::log::max_level() >= ::log::LevelFilter::Debug)
        .mount("/", routes![attach])
        .launch()
}
