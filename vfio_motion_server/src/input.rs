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
        Virt(err: ::virt::error::Error) {
            from()
            display("{}", err)
        }
    }
}

#[derive(Serialize)]
pub struct Device {
    evdev: String,
    domain: Domain,

    xml: String,
}
impl Device {
    pub fn new(domain: Domain, evdev: PathBuf) -> Result<Self, Error> {
        if !evdev.exists() {
            return Err(Error::BadEvdev(evdev));
        }

        let flags = SFlag::from_bits_truncate(stat(&evdev)?.st_mode);
        debug!("evdev {:?} st_mode: {:#x}", evdev, flags);
        if !flags.contains(SFlag::S_IFCHR) {
            return Err(Error::BadEvdev(evdev));
        }

        let evdev = evdev.to_string_lossy().into_owned();
        let mut instance = Device {
            evdev,
            domain,
            xml: String::new()
        };
        instance._xml();
        Ok(instance)
    }
    fn _xml(&mut self) {
        self.xml = format!(include_str!("attach_detach.xml"), evdev=self.evdev);
    }

    pub fn evdev(&self) -> &Path {
        &Path::new(&self.evdev)
    }
    pub fn domain(&self) -> &Domain {
        &self.domain
    }

    pub fn attach(&self) -> Result<(), Error> {
        match self.domain.attach_device_flags(&self.xml, ::virt::domain::VIR_DOMAIN_AFFECT_LIVE) {
            Ok(_) => Ok(()),
            Err(e) => Err(
                if e.code == libvirt::VIR_ERR_INTERNAL_ERROR &&
                e.message == format!("internal error: unable to execute QEMU command \'device_add\': {}: failed to get exclusive access: Device or resource busy", self.evdev) {
                Error::BadState("Device already attached!")
            } else {
                e.into()
            })
        }
    }
    pub fn detach(&self) -> Result<(), Error> {
        match self.domain.detach_device(&self.xml) {
            Ok(_) => Ok(()),
            Err(e) => Err(
                if e.code == libvirt::VIR_ERR_OPERATION_FAILED &&
                e.message == "operation failed: matching input device not found" {
                Error::BadState("Device not attached!")
            } else {
                e.into()
            })
        }
    }
}
impl<'de> Deserialize<'de> for Device {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field { Evdev, Domain }

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
                let mut domain = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Evdev => {
                            if evdev.is_some() {
                                return Err(de::Error::duplicate_field("evdev"));
                            }
                            evdev = Some(map.next_value()?);
                        }
                        Field::Domain => {
                            if domain.is_some() {
                                return Err(de::Error::duplicate_field("domain"));
                            }
                            domain = Some(map.next_value()?);
                        }
                    }
                }

                let evdev = evdev.ok_or_else(|| de::Error::missing_field("evdev"))?;
                let domain = domain.ok_or_else(|| de::Error::missing_field("domain"))?;
                Device::new(domain, evdev).map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_struct("Device", &["evdev", "domain"], DeviceVisitor)
    }
}
