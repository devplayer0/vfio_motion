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
extern crate toml;
extern crate virt;
extern crate reqwest;
extern crate widestring;
extern crate libc;
extern crate winapi;
extern crate glib_sys;
extern crate gdk_sys;
extern crate gdk;
extern crate gtk;

extern crate vfio_motion_common;

#[macro_use]
mod util;
pub mod config;
pub mod win;
mod service;
pub mod gui;

use vfio_motion_common::libvirt::Connection;
use vfio_motion_common::input::{NativeInput, HttpInput};
use config::Config;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let input_api = if config.native {
        info!("native backend, opening connection to libvirt...");
        NativeInput::new(Connection::open(&config.libvirt.uri)?)
    } else {
        info!("http backend, creating client...");
        HttpInput::new(reqwest::Client::new(), &config.http.url)
    };

    if config.is_service {
        info!("starting service...");
        service::run(&config, input_api)?;
    } else {
        info!("starting gui...");
        gui::run(&config, input_api)?;
    }

    debug!("shutting down...");
    Ok(())
}
