use std::error::Error;
use std::rc::Rc;
use std::cell::RefCell;

use ::log::{self, Log, LevelFilter};
use ::simplelog::{self, SharedLogger};
use ::gtk;
use gtk::prelude::*;
use gtk::{MessageDialog, DialogFlags, MessageType, ButtonsType};

use ::vfio_motion_common::input::{self, Input, Domains, Device};
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

struct ConfigUi<'a> {
    config: Rc<RefCell<Config>>,
    input: Box<Input + 'a>,

    save: gtk::Button,

    // General page
    libvirt_mode: gtk::ComboBox,
    domains: gtk::ListStore,
    domain: gtk::ComboBox,
    service_startup: gtk::Switch,
    shortcut: gtk::Button,
    libvirt_uri: gtk::Entry,
    http_url: gtk::Entry,
    log_dir: gtk::FileChooser,

    // Devices page
    devices: gtk::ListStore,
}
impl<'a> ConfigUi<'a> {
    pub fn new(builder: gtk::Builder, config: &Config, input: Box<Input + 'a>) -> ConfigUi<'a> {
        let save            = builder.get_object("save").unwrap();

        // General page
        let libvirt_mode    = builder.get_object("libvirt_mode").unwrap();
        let domains         = builder.get_object("domains").unwrap();
        let domain          = builder.get_object("domain").unwrap();
        let service_startup = builder.get_object("service_startup").unwrap();
        let shortcut        = builder.get_object("shortcut").unwrap();
        let libvirt_uri     = builder.get_object("libvirt_uri").unwrap();
        let http_url        = builder.get_object("http_url").unwrap();
        let log_dir         = builder.get_object("log_dir").unwrap();

        // Devices page
        let devices         = builder.get_object("devices").unwrap();

        let window: gtk::Window = builder.get_object("window").unwrap();
        window.show_all();
        window.connect_delete_event(|_, _| {
            gtk::main_quit();
            Inhibit(false)
        });

        ConfigUi {
            config: Rc::new(RefCell::new(config.clone())),
            input,

            save: save,
            // General page
            libvirt_mode, domains, domain, service_startup, shortcut, libvirt_uri, http_url, log_dir,
            // Devices page
            devices,
        }
    }

    pub fn load(&mut self) -> Result<(), input::Error> {
        let conf = self.config.borrow();

        // General page
        self.libvirt_mode.set_active_id(if conf.native {
            "native"
        } else {
            "http"
        });

        self.domains.clear();
        let mut i_dom = 0;
        for (i, dom) in self.input.domains().list()?.iter().enumerate() {
            if dom == &conf.domain {
                i_dom = i as i32;
            }
            self.domains.set_value(&self.domains.append(), 0, &gtk::Value::from(dom));
        }
        self.domain.set_active(i_dom);

        self.service_startup.set_active(conf.service_startup);
        self.libvirt_uri.set_text(&conf.libvirt.uri);
        self.http_url.set_text(&conf.http.url);
        self.log_dir.set_filename(&conf.log_dir);

        // Devices page
        self.devices.clear();
        for dev in &conf.devices {
            let tree_iter = self.devices.append();
            self.devices.set_value(&tree_iter, 0, &dev.to_value());
            self.devices.set_value(&tree_iter, 1, &self.input.device(&conf.domain, dev)?.attached().to_value());
        }

        let w_conf = Rc::downgrade(&self.config);
        let w_save = self.save.downgrade();

        // General page
        self.libvirt_mode.connect_changed(clone!(w_conf, w_save => move |lvm| {
            upgrade_weak!(w_save).set_sensitive(true);

            let conf = upgrade_weak!(w_conf);
            conf.borrow_mut().native = match lvm.get_active_id().unwrap().as_ref() {
                "native" => true,
                "http" => false,
                _ => panic!("can't happen!")
            };

            trace!("libvirt mode changed, native?: {}", conf.borrow().native);
        }));
        self.domain.connect_changed(clone!(w_conf, w_save => move |d| {
            upgrade_weak!(w_save).set_sensitive(true);

            let conf = upgrade_weak!(w_conf);
            conf.borrow_mut().domain = d.get_active_id().unwrap();

            trace!("domain changed to {}", conf.borrow().domain);
        }));
        self.service_startup.connect_state_set(clone!(w_conf, w_save => move |_, state| {
            upgrade_weak!(w_save, Inhibit(false)).set_sensitive(true);

            let conf = upgrade_weak!(w_conf, Inhibit(false));
            conf.borrow_mut().service_startup = state;

            trace!("service startup changed: {}", state);
            Inhibit(false)
        }));
        self.libvirt_uri.connect_changed(clone!(w_conf, w_save => move |lvu| {
            upgrade_weak!(w_save).set_sensitive(true);

            let conf = upgrade_weak!(w_conf);
            conf.borrow_mut().libvirt.uri = lvu.get_text().unwrap();

            trace!("libvirt uri changed to {}", conf.borrow().libvirt.uri);
        }));
        self.http_url.connect_changed(clone!(w_conf, w_save => move |hu| {
            upgrade_weak!(w_save).set_sensitive(true);

            let conf = upgrade_weak!(w_conf);
            conf.borrow_mut().http.url = hu.get_text().unwrap();

            trace!("http url changed to {}", conf.borrow().http.url);
        }));
        self.log_dir.connect_selection_changed(clone!(w_conf, w_save => move |ld| {
            upgrade_weak!(w_save).set_sensitive(true);

            let conf = upgrade_weak!(w_conf);
            conf.borrow_mut().log_dir = ld.get_filename().unwrap().to_string_lossy().to_string();

            trace!("log dir changed to {}", conf.borrow().log_dir);
        }));

        Ok(())
    }
}

pub fn run(config: &Config, input: Box<Input + '_>) -> Result<(), Box<dyn Error>> {
    let builder = gtk::Builder::new_from_string(GLADE_SRC);

    let mut ui_config = ConfigUi::new(builder, config, input);
    ui_config.load()?;
    gtk::main();

    Ok(())
}
