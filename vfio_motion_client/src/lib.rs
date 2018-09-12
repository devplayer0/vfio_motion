use std::error::Error;

#[macro_use]
extern crate quick_error;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;

extern crate simplelog;
extern crate config as config_rs;
extern crate reqwest;
extern crate widestring;
extern crate winapi;
extern crate gtk;

pub mod config;
mod input;
mod win;
mod service;
pub mod gui;

use config::Config;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    if config.is_service {
        debug!("starting service...");
        service::run(config)?;
    } else {
        debug!("starting gui...");
        gui::run(config)?;
    }

    debug!("shutting down...");
    Ok(())
}
