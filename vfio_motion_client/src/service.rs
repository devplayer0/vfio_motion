use std::error::Error as StdError;
use std::time::Duration;
use std::thread;

use ::winapi::um::winuser;
use ::winapi::um::wincon::{CTRL_C_EVENT, CTRL_CLOSE_EVENT};
use ::virt;
use ::reqwest;

use ::config::Config;
use ::win::{self, Hotkey};

use ::vfio_motion_common::libvirt::{Connection, Domain};
use ::vfio_motion_common::input::{Device, NativeDevice, HttpDevice};

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        NoDevices {
            description("no devices are configured!")
        }
    }
}

static mut MAIN_THREAD_ID: u32 = 0;

pub fn run(config: Config) -> Result<(), Box<dyn StdError>> {
    unsafe {
        MAIN_THREAD_ID = win::get_current_thread_id();
    }

    win::set_ctrl_handler(|t| {
        match t {
            CTRL_C_EVENT | CTRL_CLOSE_EVENT => {
                let thread_id = unsafe { MAIN_THREAD_ID };

                trace!("sending shutdown message");
                win::post_thread_message(thread_id, winuser::WM_DESTROY, 0, 0).unwrap();
                true
            },
            _ => false
        }
    })?;

    if config.devices.len() == 0 {
        return Err(Box::new(Error::NoDevices));
    }

    let mut virt_conn = None;
    let mut client = None;
    if config.native {
        info!("native backend, opening connection to libvirt...");
        virt_conn = Some(Connection::open(&config.libvirt.uri)?);
    } else {
        info!("http backend, creating client...");
        client = Some(reqwest::Client::new());
    }

    let mut devices: Vec<Box<Device>> = Vec::with_capacity(config.devices.len());
    for device in &config.devices {
        devices.push(if config.native {
            Box::new(NativeDevice::new(Domain::from(try!(virt::domain::Domain::lookup_by_name(virt_conn.as_ref().unwrap(), &config.domain))), device.to_string())?)
        } else {
            Box::new(HttpDevice::new(client.as_ref().unwrap(), &config.http.url, &config.domain, &device))
        });
        info!("configured evdev '{}'", device);
    }

    let hotkey = Hotkey::new(winuser::MOD_CONTROL | winuser::MOD_NOREPEAT, winuser::VK_TAB)?;
    loop {
        let msg = win::get_message(0, 0)?;
        if msg.message == winuser::WM_DESTROY {
            break;
        }

        if hotkey.matches(&msg) {
            for device in &mut devices {
                if let Err(e) = device.toggle() {
                    error!("failed to toggle device at '{}' state: {}", device.evdev(), e);
                    break;
                }
                match device.attached() {
                    true => info!("attached device at '{}' to domain '{}'", device.evdev(), device.domain()),
                    false => info!("detached device at '{}' from domain '{}'", device.evdev(), device.domain())
                };

                // sleep for a bit or we'll end up with keys stuck down
                thread::sleep(Duration::from_millis(300));
            }
        }
    }

    Ok(())
}
