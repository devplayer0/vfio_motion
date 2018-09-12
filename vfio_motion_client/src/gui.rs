use std::error::Error;

use ::log::{self, Log, LevelFilter};
use ::simplelog::{self, SharedLogger};
use ::gtk;
use gtk::prelude::*;
use gtk::{MessageDialog, DialogFlags, MessageType, ButtonsType};

use ::config::Config;

const GLADE_SRC: &'static str = include_str!("ui.glade");

pub struct MessageBoxLogger(LevelFilter);
impl MessageBoxLogger {
    pub fn new(log_level: LevelFilter) -> Box<MessageBoxLogger> {
        Box::new(MessageBoxLogger(log_level))
    }
}
impl Log for MessageBoxLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.0
    }
    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            MessageDialog::new(None::<&gtk::Window>, DialogFlags::empty(), MessageType::Error, ButtonsType::None, &format!("{}: {}", record.level(), record.args())).run();
        }
    }
    fn flush(&self) {}
}
impl SharedLogger for MessageBoxLogger {
    fn level(&self) -> LevelFilter {
        self.0
    }
    fn config(&self) -> Option<&simplelog::Config> {
        None
    }
    fn as_log(self: Box<Self>) -> Box<Log> {
        self
    }
}

struct ConfigUi {
    changed: bool,

    libvirt_mode: gtk::ComboBox,
    domains: gtk::ListStore,
    domain: gtk::ComboBox,
    service_startup: gtk::Switch,
    shortcut: gtk::Button,
    libvirt_uri: gtk::Entry,
    http_url: gtk::Entry,
    log_dir: gtk::FileChooser,
}
impl ConfigUi {
    pub fn new(builder: &gtk::Builder) -> ConfigUi {
        let libvirt_mode    = builder.get_object("libvirt_mode").unwrap();
        let domains         = builder.get_object("domains").unwrap();
        let domain          = builder.get_object("domain").unwrap();
        let service_startup = builder.get_object("service_startup").unwrap();
        let shortcut        = builder.get_object("shortcut").unwrap();
        let libvirt_uri     = builder.get_object("libvirt_uri").unwrap();
        let http_url        = builder.get_object("http_url").unwrap();
        let log_dir         = builder.get_object("log_dir").unwrap();

        ConfigUi {
            changed: false, libvirt_mode, domains, domain, service_startup, shortcut, libvirt_uri, http_url, log_dir,
        }
    }

    pub fn load(&self, config: &Config) {
        self.libvirt_mode.set_active_id(if config.native {
            "native"
        } else {
            "http"
        });
        self.service_startup.set_active(config.service_startup);
        self.libvirt_uri.set_text(&config.libvirt.uri);
        self.http_url.set_text(&config.http.url);
        self.log_dir.set_filename(&config.log_dir);
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let builder = gtk::Builder::new_from_string(GLADE_SRC);

    let ui_config = ConfigUi::new(&builder);
    ui_config.load(&config);

    let window: gtk::Window = builder.get_object("window").unwrap();
    window.show_all();
    window.connect_delete_event(move |_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    gtk::main();

    Ok(())
}
