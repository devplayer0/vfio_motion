use std::process;
use std::cmp;

#[macro_use]
extern crate log;

extern crate simplelog;
extern crate clap;
extern crate config as config_rs;

use log::LevelFilter;
use simplelog::TermLogger;
use config_rs::Config as ConfigRs;
use config_rs::ConfigError;

extern crate vfio_motion_common;
extern crate vfio_motion_client;

use vfio_motion_common::util::SingleItemSource;
use vfio_motion_client::config::Config;

fn args<'a>() -> clap::ArgMatches<'a> {
    clap::App::new("vfio-motion client")
        .version("0.1")
        .author("Jack O'Sullivan <jackos1998@gmail.com>")
        .arg(clap::Arg::with_name("config")
             .short("c")
             .long("config")
             .value_name("FILE")
             .help("Set config file path")
             .default_value("%APPDATA%/vfio-motion.toml")
             .takes_value(true))
        .arg(clap::Arg::with_name("daemon")
             .short("d")
             .long("daemon")
             .help("Start in daemon / service mode"))
        .arg(clap::Arg::with_name("v")
             .short("v")
             .multiple(true)
             .help("Print extra log messages"))
        .get_matches()
}
fn load_config(args: clap::ArgMatches) -> Result<Config, ConfigError> {
    let mut config = ConfigRs::default();
    config.set_default("log_level", LevelFilter::Info.to_string())?;
    config.set_default("libvirt.uri", "qemu+tcp://10.0.122.1/system")?;
    config.set_default("http.url", "http://127.0.0.1:3020")?;
    config.set_default("domain", "gpu")?;
    config.set_default("devices", Vec::new() as Vec<String>)?;

    config.merge(config_rs::File::with_name(args.value_of("config").unwrap()).required(false))?;

    let mut cur_config: Config = config.clone().try_into().unwrap();
    config.merge(SingleItemSource::new("log_level", cmp::max(cur_config.log_level()?, match args.occurrences_of("v") {
        0 => cur_config.log_level()?,
        1 => LevelFilter::Debug,
        2 | _ => LevelFilter::Trace,
    }).to_string()))?;

    let mut conf: Config = config.try_into()?;
    conf.is_service = args.is_present("daemon");
    Ok(conf)
}
fn main() {
    let mut config = load_config(args()).unwrap();
    TermLogger::init(config.log_level().unwrap(), simplelog::Config::default()).unwrap();

    if let Err(e) = vfio_motion_client::run(config) {
        error!("{}", e);
        process::exit(1);
    }
}
