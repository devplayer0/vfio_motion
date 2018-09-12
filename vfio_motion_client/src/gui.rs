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

pub fn run(_config: Config) -> Result<(), Box<dyn Error>> {
    let builder = gtk::Builder::new_from_string(GLADE_SRC);

    let window: gtk::Window = builder.get_object("window").unwrap();
    window.show_all();
    window.connect_delete_event(move |_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    gtk::main();

    Ok(())
}
