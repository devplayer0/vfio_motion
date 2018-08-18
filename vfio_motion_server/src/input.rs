use std::path::{Path, PathBuf};

extern crate nix;
extern crate virt;

use self::nix::sys::stat::{stat, SFlag};

use libvirt::Domain;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        SysError(err: nix::Error) {
            from()
        }
        BadEvdev(evdev: PathBuf) {
            display(me) -> ("Invalid evdev {:?}", evdev)
        }
        BadState(msg: &'static str) {
            description(msg)
        }
        Libvirt(err: virt::error::Error) {
            from()
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
        let flags = SFlag::from_bits_truncate(stat(evdev)?.st_mode);
        debug!("evdev {:?} st_mode: {:#x}", evdev, flags);
        if !evdev.exists() || !flags.contains(SFlag::S_IFCHR) {
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
        /*if let Some(_) = self.domain {
            return Err(Error::BadState("Already attached!"));
        }*/

        debug!("qemu attach response: {}", self.domain.qemu_monitor_command(format!(include_str!("attach.json"), id=self.id, device=self.evdev, addr=self.addr).as_str(), 0)?.unwrap_or(String::from("none")));
        Ok(())
    }
    pub fn detach(&self) -> Result<(), Error> {
        /*if let None = self.domain {
            return Err(Error::BadState("Not attached!"));
        }*/

        debug!("qemu detach response: {}", self.domain.qemu_monitor_command(format!(include_str!("detach.json"), id=self.id).as_str(), 0)?.unwrap_or(String::from("none")));
        Ok(())
    }
}
