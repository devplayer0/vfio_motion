use std::error::Error;
use std::rc::Rc;
use std::cell::{Cell, RefCell};
use std::ptr;
use std::fs;

use ::log::{self, Log, LevelFilter};
use ::simplelog::{self, SharedLogger};
use ::toml;
use ::libc;
use ::winapi::um::winuser;
use ::glib_sys;
use ::gdk_sys;
use ::gdk;
use gdk::prelude::*;
use gdk::enums::key;
use gdk::ModifierType;
use ::gtk;
use gtk::prelude::*;
use gtk::{MessageDialog, DialogFlags, MessageType, ButtonsType};
use ::reqwest;

use ::vfio_motion_common::libvirt::Connection;
use ::vfio_motion_common::input::{self, Input, NativeInput, HttpInput};

use ::config::Config;

const GLADE_SRC: &'static str = include_str!("ui.glade");
const MODIFIER_KEYS: [key::Key; 4] = [ key::Control_L, key::Control_R, key::Shift_L, key::Shift_R ];
pub const DEFAULT_HOTKEY: &'static str = "<Primary>Tab";

// no support for windows key in GTK on Windows :(
pub fn win_hotkey(key: key::Key, mods: ModifierType) -> Result<(isize, u32), &'static str> {
    let mut win_mods = 0;
    if mods.contains(ModifierType::MOD1_MASK) {
        win_mods |= winuser::MOD_ALT;
    }
    if mods.contains(ModifierType::CONTROL_MASK) {
        win_mods |= winuser::MOD_CONTROL;
    }
    if mods.contains(ModifierType::SHIFT_MASK) {
        win_mods |= winuser::MOD_SHIFT;
    }

    #[allow(unused_assignments)]
    let mut win_key = 0;
    unsafe {
        let dpy = gdk_sys::gdk_display_get_default();
        if dpy.is_null() {
            return Err("failed to get default gdk display");
        }
        let keymap = gdk_sys::gdk_keymap_get_for_display(dpy);
        if keymap.is_null() {
            return Err("failed to get display keymap");
        }

        let mut keys = ptr::null_mut();
        let mut n_keys = 0;
        if gdk_sys::gdk_keymap_get_entries_for_keyval(keymap, key, &mut keys, &mut n_keys) == 0 {
            return Err("failed to get keycodes for key");
        }
        assert_ne!(n_keys, 0);

        win_key = (*keys).keycode;
        glib_sys::g_free(keys as *mut libc::c_void);
    }

    Ok((win_mods, win_key))
}

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
            let dialog = MessageDialog::new(None::<&gtk::Window>, DialogFlags::empty(), MessageType::Error, ButtonsType::Close, &format!("{}: {}", record.level(), record.args()));
            dialog.run();
            dialog.destroy();
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
    config: Rc<RefCell<Config>>,
    conn_changed: Rc<Cell<bool>>,
    input: Rc<RefCell<Option<Box<Input>>>>,
    connect_and_reload: Rc<RefCell<Option<Box<Fn() -> bool>>>>,

    window: gtk::Window,
    save: gtk::Button,
    save_notification: gtk::InfoBar,

    // General page
    libvirt_mode: gtk::ComboBox,
    domains: gtk::ListStore,
    domain: gtk::ComboBox,
    service_startup: gtk::Switch,
    hotkey: gtk::Button,
    libvirt_uri: gtk::Entry,
    http_url: gtk::Entry,
    log_dir: gtk::FileChooser,

    // Devices page
    devices: gtk::ListStore,
}
impl ConfigUi {
    pub fn new(builder: gtk::Builder, config: &Config) -> ConfigUi {
        let save                = builder.get_object("save").unwrap();
        let save_notification   = builder.get_object("save_notification").unwrap();

        // General page
        let libvirt_mode        = builder.get_object("libvirt_mode").unwrap();
        let domains             = builder.get_object("domains").unwrap();
        let domain              = builder.get_object("domain").unwrap();
        let service_startup     = builder.get_object("service_startup").unwrap();
        let hotkey              = builder.get_object("hotkey").unwrap();
        let libvirt_uri         = builder.get_object("libvirt_uri").unwrap();
        let http_url            = builder.get_object("http_url").unwrap();
        let log_dir             = builder.get_object("log_dir").unwrap();

        // Devices page
        let devices             = builder.get_object("devices").unwrap();

        let window: gtk::Window = builder.get_object("window").unwrap();
        window.show_all();
        window.connect_delete_event(|_, _| {
            gtk::main_quit();
            Inhibit(false)
        });

        ConfigUi {
            config: Rc::new(RefCell::new(config.clone())),
            conn_changed: Rc::new(Cell::new(false)),
            input: Rc::new(RefCell::new(None)),
            connect_and_reload: Rc::new(RefCell::new(None)),

            window, save, save_notification,
            // General page
            libvirt_mode, domains, domain, service_startup, hotkey, libvirt_uri, http_url, log_dir,
            // Devices page
            devices,
        }
    }

    pub fn load(&mut self) -> Result<(), input::Error> {
        let w_conf = Rc::downgrade(&self.config);

        let w_input = Rc::downgrade(&self.input);
        let w_domains = self.domains.downgrade();
        let w_domain = self.domain.downgrade();
        self.connect_and_reload.replace(Some(Box::new(clone!(w_conf, w_input, w_domains, w_domain => move || {
            let conf = upgrade_weak!(w_conf, false);
            let input = upgrade_weak!(w_input, false);
            let domains = upgrade_weak!(w_domains, false);
            let domain = upgrade_weak!(w_domain, false);

            input.replace(if conf.borrow().native {
                info!("native backend, opening connection to libvirt...");
                let uri = &conf.borrow().libvirt.uri;
                let conn = match Connection::open(uri) {
                    Ok(c) => Some(c),
                    Err(e) => {
                        error!("failed to open connection to libvirtd on {}: {}", uri, e);
                        None
                    }
                };
                match conn {
                    Some(c) => Some(NativeInput::new(c)),
                    None => None
                }
            } else {
                info!("http backend, creating client...");
                Some(HttpInput::new(reqwest::Client::new(), &conf.borrow().http.url))
            });

            let input = &*input.borrow();
            domains.clear();
            if let Some(i) = input {
                match i.domains().list() {
                    Ok(doms) => {
                        let mut i_dom = 0;
                        for (i, dom) in doms.iter().enumerate() {
                            if dom == &conf.borrow().domain {
                                i_dom = i;
                            }
                            domains.set_value(&domains.append(), 0, &gtk::Value::from(dom));
                        }
                        domain.set_sensitive(true);
                        domain.set_active(i_dom as i32);
                    },
                    Err(e) => {
                        error!("failed to retrieve domain list: {}", e);
                        domain.set_sensitive(false);
                        return false;
                    }
                }
            } else {
                domain.set_sensitive(false);
                return false;
            }
            true
        }))));

        // General page
        self.connect_and_reload.borrow().as_ref().unwrap()();

        let mut conf = self.config.borrow_mut();
        self.libvirt_mode.set_active_id(if conf.native {
            "native"
        } else {
            "http"
        });

        self.service_startup.set_active(conf.service_startup);
        self.libvirt_uri.set_text(&conf.libvirt.uri);
        self.http_url.set_text(&conf.http.url);
        self.log_dir.set_filename(&conf.logging.dir);

        {
            let (mut h_key, mut h_mod) = gtk::accelerator_parse(&conf.hotkey);
            if h_mod == ModifierType::empty() && h_key == 0 {
                conf.hotkey = DEFAULT_HOTKEY.to_owned();
                h_mod = ModifierType::CONTROL_MASK;
                h_key = key::Tab;

                self.save.set_sensitive(true);
                error!("failed to parse hotkey '{}', using default...", conf.hotkey);
            }
            self.hotkey.set_label(&gtk::accelerator_get_label(h_key, h_mod).unwrap());
        }

        // Devices page
        self.devices.clear();
        for dev in &conf.devices {
            let tree_iter = self.devices.append();
            self.devices.set_value(&tree_iter, 0, &dev.to_value());

            let attached = match *self.input.borrow() {
                Some(ref i) => match i.device(&conf.domain, dev) {
                    Ok(ref d) => {
                        d.attached()
                    },
                    Err(_) => false
                },
                None => false
            }.to_value();
            self.devices.set_value(&tree_iter, 1, &attached);
        }

        self.save_notification.set_default_response(gtk::ResponseType::Close.into());

        let w_window = self.window.downgrade();
        let w_save = self.save.downgrade();
        let w_c_reload = Rc::downgrade(&self.connect_and_reload);
        let w_c_changed = Rc::downgrade(&self.conn_changed);

        // General page
        self.libvirt_mode.connect_changed(clone!(w_conf, w_save, w_c_reload => move |lvm| {
            let conf = upgrade_weak!(w_conf);
            let old_state = conf.borrow().native;
            conf.borrow_mut().native = match lvm.get_active_id().unwrap().as_ref() {
                "native" => true,
                "http" => false,
                _ => panic!("can't happen!")
            };

            let c_reload = upgrade_weak!(w_c_reload);
            if !c_reload.borrow().as_ref().unwrap()() {
                conf.borrow_mut().native = old_state;
                lvm.set_active_id(match old_state {
                    true => "native",
                    false => "http"
                });
                return;
            }

            if conf.borrow().native != old_state {
                upgrade_weak!(w_save).set_sensitive(true);
            }
            debug!("libvirt mode changed, native?: {}", conf.borrow().native);
        }));
        self.domain.connect_changed(clone!(w_conf, w_save => move |d| {
            if let None = d.get_active_id() {
                return;
            }

            let conf = upgrade_weak!(w_conf);
            let old = conf.borrow().domain.clone();
            conf.borrow_mut().domain = d.get_active_id().unwrap();

            if conf.borrow().domain != old {
                upgrade_weak!(w_save).set_sensitive(true);
            }
            debug!("domain changed to {}", conf.borrow().domain);
        }));
        self.service_startup.connect_state_set(clone!(w_conf, w_save => move |_, state| {
            let conf = upgrade_weak!(w_conf, Inhibit(false));
            conf.borrow_mut().service_startup = state;

            upgrade_weak!(w_save, Inhibit(false)).set_sensitive(true);
            debug!("service startup changed: {}", state);
            Inhibit(false)
        }));
        self.hotkey.connect_clicked(clone!(w_conf, w_save, w_window => move |h| {
            let conf = upgrade_weak!(w_conf);
            let dpy = h.get_display().unwrap();
            let kb = dpy.get_default_seat().unwrap().get_keyboard().unwrap();
            let w_kb = kb.downgrade();

            #[allow(deprecated)]
            kb.grab(&dpy.get_default_screen().get_root_window().unwrap(),
                    gdk::GrabOwnership::Window,
                    true,
                    gdk::EventMask::KEY_PRESS_MASK | gdk::EventMask::KEY_RELEASE_MASK,
                    None,
                    gdk_sys::GDK_CURRENT_TIME as u32);

            let hotkey_accel = Rc::new(RefCell::new(String::default()));
            let w_hk_accel = Rc::downgrade(&hotkey_accel);

            let dialog = MessageDialog::new(Some(&upgrade_weak!(w_window)), DialogFlags::DESTROY_WITH_PARENT, MessageType::Question, ButtonsType::Cancel, "Enter a keyboard shortcut");
            dialog.connect_key_press_event(clone!(w_hk_accel => move |_s, e| {
                let name = gtk::accelerator_name(e.get_keyval(), e.get_state()).unwrap();
                debug!("hotkey accel name: {}", name);
                upgrade_weak!(w_hk_accel, Inhibit(false)).replace(name);

                Inhibit(false)
            }));
            dialog.connect_key_release_event(clone!(w_kb => move |d, e| {
                if MODIFIER_KEYS.contains(&e.get_keyval()) {
                    return Inhibit(false);
                }

                #[allow(deprecated)]
                upgrade_weak!(w_kb, Inhibit(false)).ungrab(gdk_sys::GDK_CURRENT_TIME as u32);
                d.response(gtk::ResponseType::Apply.into());
                Inhibit(false)
            }));

            if gtk::ResponseType::from(dialog.run()) == gtk::ResponseType::Apply && hotkey_accel.borrow().as_str() != conf.borrow().hotkey.as_str() {
                conf.borrow_mut().hotkey = hotkey_accel.borrow().clone();
                let (h_key, h_mod) = gtk::accelerator_parse(conf.borrow().hotkey.as_str());
                h.set_label(&gtk::accelerator_get_label(h_key, h_mod).unwrap());

                upgrade_weak!(w_save).set_sensitive(true);
                debug!("hotkey changed to {}", conf.borrow().hotkey);
            }
            dialog.destroy();

            #[allow(deprecated)]
            kb.ungrab(gdk_sys::GDK_CURRENT_TIME as u32);
        }));
        self.libvirt_uri.connect_changed(clone!(w_conf, w_save, w_c_changed => move |lvu| {
            let conf = upgrade_weak!(w_conf);
            let old_uri = conf.borrow().libvirt.uri.clone();
            conf.borrow_mut().libvirt.uri = lvu.get_text().unwrap();

            if conf.borrow().libvirt.uri != old_uri {
                upgrade_weak!(w_c_changed).set(true);
            }
            upgrade_weak!(w_save).set_sensitive(true);
            debug!("libvirt uri changed to {}", conf.borrow().libvirt.uri);
        }));
        self.http_url.connect_changed(clone!(w_conf, w_save, w_c_changed => move |hu| {
            let conf = upgrade_weak!(w_conf);
            let old_url = conf.borrow().http.url.clone();
            conf.borrow_mut().http.url = hu.get_text().unwrap();

            if conf.borrow().http.url != old_url {
                upgrade_weak!(w_c_changed).set(true);
            }
            upgrade_weak!(w_save).set_sensitive(true);
            debug!("http url changed to {}", conf.borrow().http.url);
        }));
        self.log_dir.connect_selection_changed(clone!(w_conf, w_save => move |ld| {
            let conf = upgrade_weak!(w_conf);
            let new_dir = ld.get_filename().unwrap().to_string_lossy().to_string();
            if new_dir == conf.borrow().logging.dir {
                return;
            }
            conf.borrow_mut().logging.dir = new_dir;

            upgrade_weak!(w_save).set_sensitive(true);
            debug!("log dir changed to {}", conf.borrow().logging.dir);
        }));

        let w_save_notif = self.save_notification.downgrade();
        self.save.connect_clicked(clone!(w_conf, w_save_notif, w_c_reload, w_c_changed => move |s| {
            if upgrade_weak!(w_c_changed).get() {
                let c_reload = upgrade_weak!(w_c_reload);
                if !c_reload.borrow().as_ref().unwrap()() {
                    return;
                }
            }

            let conf = upgrade_weak!(w_conf);
            let conf_str = match toml::to_string(&*conf.borrow()) {
                Ok(c) => c,
                Err(e) => {
                    error!("failed to serialize configuration: {}", e);
                    return;
                }
            };
            if let Err(e) = fs::write(conf.borrow().file(), conf_str) {
                error!("failed to write configuration to '{}': {}", conf.borrow().file(), e);
                return;
            }

            s.set_sensitive(false);
            upgrade_weak!(w_save_notif).set_revealed(true);
            gtk::timeout_add_seconds(2, clone!(w_save_notif => move || {
                upgrade_weak!(w_save_notif, Continue(false)).set_revealed(false);
                Continue(false)
            }));
            info!("configuration written to {}", conf.borrow().file());
        }));
        self.save_notification.connect_close(|sn| sn.set_revealed(false));
        self.save_notification.connect_response(|sn, res| if gtk::ResponseType::from(res) == gtk::ResponseType::Close {
            sn.emit_close();
        });

        Ok(())
    }
}

pub fn run(config: &Config) -> Result<(), Box<dyn Error>> {
    let builder = gtk::Builder::new_from_string(GLADE_SRC);

    let mut ui_config = ConfigUi::new(builder, config);
    ui_config.load()?;
    gtk::main();

    Ok(())
}
