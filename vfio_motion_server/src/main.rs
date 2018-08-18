extern crate clap;
extern crate config;

extern crate vfio_motion_server;

use std::process;

use config::Config;

fn main() {
    let args = clap::App::new("vfio-motion server")
        .version("0.1")
        .author("Jack O'Sullivan <jackos1998@gmail.com>")
        .arg(clap::Arg::with_name("config")
             .short("c")
             .long("config")
             .value_name("FILE")
             .help("Set config file path")
             .default_value("/etc/virtio-motion.yaml")
             .takes_value(true))
        .arg(clap::Arg::with_name("v")
             .short("v")
             .help("Print extra log messages"))
        .get_matches();

    let mut config = Config::default();
    config
        .merge(config::File::with_name(args.value_of("config").unwrap())).unwrap()
        .merge(config::Environment::with_prefix("VFIO_MOTION")).unwrap();

    if let Err(e) = vfio_motion_server::run(config) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
