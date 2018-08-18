use std::cmp;
use std::process;

#[macro_use]
extern crate log;
extern crate simplelog;
extern crate clap;
extern crate config;

use log::LevelFilter;
use simplelog::TermLogger;

extern crate vfio_motion_server;
use vfio_motion_server::Config;
use vfio_motion_server::util::SingleItemSource;

fn args<'a>() -> clap::ArgMatches<'a> {
    clap::App::new("vfio-motion server")
        .version("0.1")
        .author("Jack O'Sullivan <jackos1998@gmail.com>")
        .arg(clap::Arg::with_name("config")
             .short("c")
             .long("config")
             .value_name("FILE")
             .help("Set config file path")
             .default_value("/etc/virtio-motion.toml")
             .takes_value(true))
        .arg(clap::Arg::with_name("v")
             .short("v")
             .multiple(true)
             .help("Print extra log messages"))
        .get_matches()
}
fn load_config(args: clap::ArgMatches) -> Result<Config, config::ConfigError> {
    let mut config = config::Config::default();
    config.set_default("log_level", LevelFilter::Info.to_string())?;
    config.set_default("libvirt_uri", "qemu:///system")?;

    config.merge(config::File::with_name(args.value_of("config").unwrap()))?;
    let mut cur_config: Config = config.clone().try_into().unwrap();
    config.merge(SingleItemSource::from("log_level", cmp::max(cur_config.log_level()?, match args.occurrences_of("v") {
        0 => cur_config.log_level()?,
        1 => LevelFilter::Debug,
        2 | _ => LevelFilter::Trace,
    }).to_string()))?;
    config.merge(config::Environment::with_prefix("VFIO_MOTION"))?;

    config.try_into()
}

fn main() {
    let mut config = load_config(args()).unwrap();
    TermLogger::init(config.log_level().unwrap(), simplelog::Config::default()).unwrap();

    trace!("log level: {}", log::max_level());
    if let Err(e) = vfio_motion_server::run(config) {
        error!("{}", e);
        process::exit(1);
    }
}
