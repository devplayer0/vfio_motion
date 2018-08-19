use std::path::{Path, PathBuf};
use std::fmt;

use ::nix::sys::stat::{stat, SFlag};
use ::serde::de::{self, Deserialize, Deserializer, Visitor, MapAccess};

use libvirt;
use libvirt::Domain;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        SysError(err: ::nix::Error) {
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

pub struct Device {
    id: String,

    evdev: PathBuf,
    addr: u32,
    domain: Domain
}
impl Device {
    pub fn new(domain: Domain, evdev: PathBuf, addr: u32) -> Result<Self, Error> {
        if !evdev.exists() {
            return Err(Error::BadEvdev(evdev));
        }

        let flags = SFlag::from_bits_truncate(stat(&evdev)?.st_mode);
        debug!("evdev {:?} st_mode: {:#x}", evdev, flags);
        if !flags.contains(SFlag::S_IFCHR) {
            return Err(Error::BadEvdev(evdev));
        }

        Ok(Device {
            id: evdev.file_name().unwrap().to_string_lossy().to_string(),
            evdev,
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
impl<'de> Deserialize<'de> for Device {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field { Evdev, Addr, Domain }

        struct DeviceVisitor;
        impl<'de> Visitor<'de> for DeviceVisitor {
            type Value = Device;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Device")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Device, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut evdev = None;
                let mut addr = None;
                let mut domain: Some<Domain> = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Evdev => {
                            if evdev.is_some() {
                                return de::Error::duplicate_field("evdev");
                            }
                            evdev = Some(map.next_value()?);
                        }
                        Field::Addr => {
                            if addr.is_some() {
                                return de::Error::duplicate_field("addr");
                            }
                            addr = Some(map.next_value()?);
                        }
                        Field::Domain => {
                            if domain.is_some() {
                                return de::Error::duplicate_field("domain");
                            }

                            let domain_name = map.next_value()?;
                            let conn = ::libvirt::Connection::open("qemu:///system")?;
                            let domain = Some(::virt::Domain::lookup_by_name(&conn, domain_name)?);
                        }
                    }
                }

                let evdev = evdev.ok_or_else(|| de::Error::missing_field("evdev"))?;
                let addr = addr.ok_or_else(|| de::Error::missing_field("addr"))?;
                let domain = domain.ok_or_else(|| de::Error::missing_field("domain"))?;
                Device::new(domain, evdev, addr)
            }
        }

        deserializer.deserialize_struct("Device", &["evdev", "addr", "domain"], DeviceVisitor)
    }
}
