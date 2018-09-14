use std::error::Error as StdError;
use std::time::Duration;
use std::thread;

use ::winapi::um::winuser;
use ::winapi::um::wincon::{CTRL_C_EVENT, CTRL_CLOSE_EVENT};

use ::config::Config;
use ::win::{self, Hotkey};

use ::vfio_motion_common::input::{Input, Device};

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        NoDevices {
            description("no devices are configured!")
        }
    }
}

static mut MAIN_THREAD_ID: u32 = 0;

pub fn run(config: &Config, input_api: Box<Input + '_>) -> Result<(), Box<dyn StdError>> {
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

    let mut devices: Vec<Box<Device>> = Vec::with_capacity(config.devices.len());
    for device in &config.devices {
        devices.push(input_api.device(&config.domain, device)?);
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
                match device.toggle() {
                    Ok(a) => if !a {
                        info!("attached device at '{}' to domain '{}'", device.evdev(), device.domain());
                    } else {
                        info!("detached device at '{}' from domain '{}'", device.evdev(), device.domain());
                    }
                    Err(e) => {
                        error!("failed to toggle device at '{}' state: {}", device.evdev(), e);
                        break;
                    }
                }

                // sleep for a bit or we'll end up with keys stuck down
                thread::sleep(Duration::from_millis(300));
            }
        }
    }

    Ok(())
}
