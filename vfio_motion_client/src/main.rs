use std::process;

#[macro_use]
extern crate log;

extern crate simplelog;

use simplelog::TermLogger;

extern crate vfio_motion_client;

use vfio_motion_client::config::Config;

fn main() {
    let config = Config::default();
    TermLogger::init(config.log_level(), simplelog::Config::default()).unwrap();

    if let Err(e) = vfio_motion_client::run(config) {
        error!("{}", e);
        process::exit(1);
    }
}
