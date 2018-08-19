use std::process;
use std::error::Error;
use std::path::Path;

#[macro_use]
extern crate quick_error;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate virt;
extern crate simple_signal;

use simple_signal::Signal;

pub mod util;
pub mod config;
mod libvirt;
mod input;
mod server;

use config::Config;

fn dummy_handler(_ctx: Box<Option<String>>, err: virt::error::Error) {
    trace!("libvirt error: {}", err);
}
pub fn run(config: Config) -> Result<(), Box<dyn Error>> {

    // Prevent libvirt built-in error logging
    libvirt::set_error_handler(Box::new(None), dummy_handler);

    let conn = libvirt::Connection::open(config.libvirt_uri())?;
    simple_signal::set_handler(&[Signal::Int, Signal::Term], |_signals| {
        info!("shutting down...");
        // TODO: find a way to do this across threads... (maybe have a dedicated libvirt thread?)
        //drop(conn);
        process::exit(0);
    });
    debug!("Opened connection to libvirt on '{}'", conn.get_uri()?);

    let domains = conn.list_all_domains(virt::connect::VIR_CONNECT_LIST_DOMAINS_ACTIVE)?;
    for domain in domains.iter().map(|d| libvirt::Domain::from(d)) {
        info!("libvirt domain: {}", domain.get_name()?);
        //input::Device::new(&domain, Path::new("/dev/input/by-id/usb-Logitech_G203_Prodigy_Gaming_Mouse_0487365B3837-event-mouse"), 0x10)?.attach()?;
        //info!("attached device!");
    }

    Err(Box::new(server::run(config.http().get())))
}
