#![cfg_attr(build = "release", windows_subsystem = "windows")]

#[cfg(build = "release")]
use std::ops::Deref;
use std::cmp;
use std::process;
use std::env;
use std::fs::{self, File};
use std::path::PathBuf;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

extern crate simplelog;
extern crate clap;
extern crate config as config_rs;
extern crate gtk;

use log::LevelFilter;
use simplelog::{CombinedLogger, WriteLogger};
use config_rs::Config as ConfigRs;
use config_rs::ConfigError;

extern crate vfio_motion_common;
extern crate vfio_motion_client;

#[cfg(build = "release")]
use vfio_motion_client::win::error_mbox;
use vfio_motion_common::util::SingleItemSource;
use vfio_motion_client::config::Config;
#[cfg(build = "release")]
use vfio_motion_client::gui::MessageBoxLogger;

#[cfg(build = "debug")]
const DEFAULT_LOG_LEVEL: LevelFilter = LevelFilter::Debug;
#[cfg(build = "release")]
const DEFAULT_LOG_LEVEL: LevelFilter = LevelFilter::Info;

lazy_static! {
    static ref DEFAULT_DIR: PathBuf = PathBuf::from(env::var("APPDATA").unwrap()).join("vfio_motion");
    static ref DEFAULT_CONFIG_FILE: String = DEFAULT_DIR.join("vfio_motion").to_string_lossy().to_string();
}

fn args<'a>() -> clap::ArgMatches<'a> {
    clap::App::new("vfio-motion client")
        .version("0.1")
        .author("Jack O'Sullivan <jackos1998@gmail.com>")
        .arg(clap::Arg::with_name("config")
             .short("c")
             .long("config")
             .value_name("FILE")
             .help("Set config file path")
             .default_value(&DEFAULT_CONFIG_FILE)
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
    config.set_default("log_level", DEFAULT_LOG_LEVEL.to_string())?;
    config.set_default("log_dir", DEFAULT_DIR.to_str().unwrap())?;
    config.set_default("native", true)?;
    config.set_default("libvirt.uri", "qemu+tcp://10.0.122.1/system")?;
    config.set_default("http.url", "http://127.0.0.1:3020")?;
    config.set_default("domain", "gpu")?;
    config.set_default("devices", Vec::new() as Vec<String>)?;
    config.set_default("service_startup", false)?;

    config.merge(config_rs::File::with_name(args.value_of("config").unwrap()).required(false))?;

    let mut cur_config: Config = config.clone().try_into()?;
    config.merge(SingleItemSource::new("log_level", cmp::max(cur_config.log_level()?, match args.occurrences_of("v") {
        0 => cur_config.log_level()?,
        1 => LevelFilter::Debug,
        2 | _ => LevelFilter::Trace,
    }).to_string()))?;

    let mut conf: Config = config.try_into()?;
    conf.is_service = args.is_present("daemon");
    Ok(conf)
}
fn configure() -> Result<Config, Box<dyn std::error::Error>> {
    let mut config = load_config(args())?;
    fs::create_dir_all(&config.log_dir)?;

    let log_level = config.log_level()?;

    CombinedLogger::init(vec![
        WriteLogger::new(log_level, simplelog::Config::default(), File::create(config.log_file())?),
        #[cfg(build = "release")]
        MessageBoxLogger::new(LevelFilter::Error),
        #[cfg(build = "debug")]
        simplelog::TermLogger::new(log_level, simplelog::Config::default()).unwrap(),
    ])?;

    Ok(config)
}

fn main() {
    #[cfg(build = "release")]
    std::panic::set_hook(Box::new(|info| {
        let (filename, line) =
            info.location().map(|loc| (loc.file(), loc.line()))
                .unwrap_or(("<unknown>", 0));

        let cause = info.payload().downcast_ref::<String>().map(String::deref);
        let cause = cause.unwrap_or_else(|| info.payload().downcast_ref::<&str>().map(|s| *s).unwrap_or("<cause unknown>"));

        error_mbox(&format!("Panic occurred at {}:{}: {}", filename, line, cause));
    }));

    gtk::init().unwrap();

    let config = match configure() {
        Ok(c) => c,
        Err(e) => {
            error!("{}", e);
            process::exit(1);
        }
    };

    if let Err(e) = vfio_motion_client::run(config) {
        error!("{}", e);
        process::exit(1);
    }
}
