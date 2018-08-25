use std::error::Error;
use std::time::Duration;
use std::thread;

#[macro_use]
extern crate quick_error;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

extern crate reqwest;
extern crate widestring;
extern crate winapi;

use winapi::um::winuser;
use winapi::um::wincon::{CTRL_C_EVENT, CTRL_CLOSE_EVENT};

pub mod config;
mod input;
mod win;

use config::Config;
use input::Device;
use win::Hotkey;

static mut MAIN_THREAD_ID: u32 = 0;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
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

    let client = reqwest::Client::new();

    let mut devices = Vec::with_capacity(config.devices().len());
    for device in config.devices() {
        debug!("configured evdev '{}'", device);
        devices.push(Device::new(&client, config.host(), config.domain(), device));
    }

    let hotkey = Hotkey::new(winuser::MOD_CONTROL | winuser::MOD_NOREPEAT, winuser::VK_TAB)?;
    loop {
        let msg = win::get_message(0, 0)?;
        if msg.message == winuser::WM_DESTROY {
            break;
        }

        if hotkey.matches(&msg) {
            for device in &mut devices {
                match device.attached() {
                    false => info!("attaching device at '{}' to domain '{}'", device.evdev(), device.domain()),
                    true => info!("detaching device at '{}' from domain '{}'", device.evdev(), device.domain())
                };
                if let Err(e) = device.toggle() {
                    error!("failed to toggle device at '{}' state: {}", device.evdev(), e);
                    break;
                }

                // sleep for a bit or we'll end up with keys stuck down
                thread::sleep(Duration::from_millis(300));
            }
        }
    }

    info!("shutting down...");
    Ok(())
}
