use std::cmp;
use std::process;

#[macro_use]
extern crate log;

extern crate simplelog;
extern crate clap;
extern crate config as config_rs;

use log::LevelFilter;
use simplelog::TermLogger;
use config_rs::Config as ConfigRs;
use config_rs::ConfigError;

#[macro_use]
extern crate vfio_motion_common;
extern crate vfio_motion_server;
use vfio_motion_common::util::SingleItemSource;
use vfio_motion_server::config::Config;

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
        .arg(clap::Arg::with_name("libvirt_uri")
             .long("qemu-uri")
             .value_name("URI")
             .help("Set libvirt URI")
             .takes_value(true))
        .arg(clap::Arg::with_name("http.port")
             .short("p")
             .long("port")
             .value_name("PORT")
             .help("Set bind port")
             .takes_value(true))
        .arg(clap::Arg::with_name("http.address")
             .short("b")
             .long("bind-host")
             .value_name("ADDRESS")
             .help("Set bind address")
             .takes_value(true))
        .get_matches()
}
fn load_config(args: clap::ArgMatches) -> Result<Config, ConfigError> {
    let mut config = ConfigRs::default();
    config.set_default("log_level", LevelFilter::Info.to_string())?;
    config.set_default("libvirt_uri", "qemu:///system")?;
    config.set_default("http.address", "127.0.0.1")?;
    config.set_default("http.port", 3020)?;


    config.merge(config_rs::File::with_name(args.value_of("config").unwrap()).required(false))?;

    merge_arg!(args, config, "libvirt_uri");
    merge_arg!(args, config, "http.address");
    merge_arg!(args, config, "http.port");
    let mut cur_config: Config = config.clone().try_into().unwrap();
    config.merge(SingleItemSource::new("log_level", cmp::max(cur_config.log_level()?, match args.occurrences_of("v") {
        0 => cur_config.log_level()?,
        1 => LevelFilter::Debug,
        2 | _ => LevelFilter::Trace,
    }).to_string()))?;

    config.merge(config_rs::Environment::with_prefix("VFIO_MOTION"))?;


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
