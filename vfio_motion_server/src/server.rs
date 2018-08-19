use ::rocket::config::Config;
use ::rocket::error::{LaunchError};
use ::rocket::response::status;

#[get("/")]
fn attach() -> status::NoContent {
    status::NoContent
}

pub fn run(config: Config) -> LaunchError {
    // Unfortunately since were using the same log framework as Rocket, log to false has no effect
    ::rocket::custom(config, ::log::max_level() >= ::log::LevelFilter::Debug)
        .mount("/", routes![attach])
        .launch()
}
