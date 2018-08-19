use std::path::{Path, PathBuf};

extern crate nix;
extern crate virt;

use self::nix::sys::stat::{stat, SFlag};

use libvirt;
use libvirt::Domain;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        SysError(err: nix::Error) {
            from()
            display("System error: {}", ::std::error::Error::description(err))
        }
        BadEvdev(evdev: PathBuf) {
            display("Invalid evdev {:?}", evdev)
        }
        BadState(msg: &'static str) {
            description(msg)
        }
        Libvirt(err: libvirt::Error) {
            from()
            display("{}", err)
        }
    }
}

pub struct Device<'a> {
    id: &'a str,
    evdev: &'a str,
    addr: u32,
    domain: &'a Domain<'a>
}
impl<'a> Device<'a> {
    pub fn new(domain: &'a Domain, evdev: &'a Path, addr: u32) -> Result<Self, Error> {
        if !evdev.exists() {
            return Err(Error::BadEvdev(evdev.to_path_buf()));
        }

        let flags = SFlag::from_bits_truncate(stat(evdev)?.st_mode);
        debug!("evdev {:?} st_mode: {:#x}", evdev, flags);
        if !flags.contains(SFlag::S_IFCHR) {
            return Err(Error::BadEvdev(evdev.to_path_buf()));
        }

        Ok(Device {
            id: evdev.file_name().unwrap().to_str().unwrap(),
            evdev: evdev.to_str().unwrap(),
            addr,
            domain
        })
    }

    pub fn attach(&self) -> Result<(), Error> {
        let msg = match self.domain.qemu_monitor_command(
            format!(include_str!("attach.json"), id=self.id, device=self.evdev, addr=self.addr).as_str(),
            libvirt::VIR_DOMAIN_QEMU_MONITOR_COMMAND_DEFAULT) {
            Ok(m) => m,
            Err(e) => return Err(if let libvirt::Error::QemuMonitor(msg) = e {
                if msg["class"] == json!("GenericError") && msg["desc"] == json!(format!("Duplicate ID '{}' for device", self.id)) {
                    Error::BadState("Device already attached!")
                } else {
                    libvirt::Error::QemuMonitor(msg).into()
                }
            } else {
                e.into()
            })
        };

        debug!("qemu attach response: {:#?}", msg.unwrap_or(json!(null)));
        Ok(())
    }
    pub fn detach(&self) -> Result<(), Error> {
        let msg = match self.domain.qemu_monitor_command(
            format!(include_str!("detach.json"), id=self.id).as_str(),
            libvirt::VIR_DOMAIN_QEMU_MONITOR_COMMAND_DEFAULT) {
            Ok(m) => m,
            Err(e) => return Err(if let libvirt::Error::QemuMonitor(msg) = e {
                if msg["class"] == json!("DeviceNotFound") {
                    Error::BadState("Device not attached!")
                } else {
                    libvirt::Error::QemuMonitor(msg).into()
                }
            } else {
                e.into()
            })
        };

        debug!("qemu detach response: {:#?}", msg.unwrap_or(json!(null)));
        Ok(())
    }
}
