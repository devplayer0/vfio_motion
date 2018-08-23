#![feature(extern_prelude)]
#![feature(plugin)]
#![plugin(rocket_codegen)]
use std::process;
use std::error::Error;

#[macro_use]
extern crate quick_error;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

extern crate config as config_rs;
extern crate serde;
extern crate virt;
extern crate nix;
extern crate simple_signal;
extern crate rocket;
extern crate rocket_contrib;

extern crate vfio_motion_common;

use simple_signal::Signal;

pub mod util;
pub mod config;
mod input;
mod server;

use ::vfio_motion_common::libvirt;
use config::Config;

fn dummy_virt_handler(_ctx: Box<Option<String>>, err: virt::error::Error) {
    trace!("libvirt error: {}", err);
}
pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    // Prevent libvirt built-in error logging
    libvirt::set_error_handler(Box::new(None), dummy_virt_handler);

    unsafe {
        input::open_global_conn(config.libvirt_uri().into())?
    }
    let conn = libvirt::Connection::open(config.libvirt_uri())?;
    simple_signal::set_handler(&[Signal::Int, Signal::Term], |_signals| {
        info!("shutting down...");
        unsafe {
            match input::close_global_conn() {
                Err(e) => error!("failed to close global connection: {}", e),
                _ => process::exit(-1)
            };
        }
        process::exit(0);
    });
    debug!("Opened connection to libvirt on '{}'", conn.get_uri()?);

    Err(Box::new(server::run(config.http().get())))
}
