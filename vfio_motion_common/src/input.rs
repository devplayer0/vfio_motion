#[cfg(target_os = "linux")]
use std::path::PathBuf;
use std::fmt;

#[cfg(target_os = "linux")]
use ::nix::sys::stat::{stat, SFlag};
use ::serde::de::{self, Deserialize, Deserializer, Visitor, MapAccess};
use ::virt::domain::{VIR_DOMAIN_AFFECT_LIVE, VIR_DOMAIN_NONE};
#[cfg(target_os = "windows")]
use ::reqwest;

use ::libvirt::{self, Connection, Domain};

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        BadEvdev(evdev: String) {
            display("Invalid evdev {:?}", evdev)
        }
        BadState(msg: &'static str) {
            description(msg)
        }
        GlobalConnNotOpen
        Libvirt(err: libvirt::Error) {
            from()
            display("{}", err)
        }
        Virt(err: ::virt::error::Error) {
            from()
            display("{}", err)
        }
        Reqwest(err: String) {
            from()
            display("http error: {}", err)
        }
    }
}

pub trait Input {
    fn domains(&self) -> Box<Domains + '_>;
    fn device<'a>(&'a self, domain: &'a str, evdev: &'a str) -> Result<Box<Device + '_>, Error>;
}

pub struct NativeInput(Connection);
impl NativeInput {
    pub fn new<'a>(conn: Connection) -> Box<Input + 'a> {
        Box::new(NativeInput(conn))
    }
}
impl Input for NativeInput {
    fn domains(&self) -> Box<Domains + '_> {
        Box::new(NativeDomains::new(&self.0))
    }
    fn device(&self, domain: &str, evdev: &str) -> Result<Box<Device + '_>, Error> {
        let dom = Domain::from(::virt::domain::Domain::lookup_by_name(&self.0, domain)?);
        Ok(Box::new(NativeDevice::new(dom, evdev.to_string())?))
    }
}

#[cfg(target_os = "windows")]
pub struct HttpInput {
    client: reqwest::Client,
    host: String,
}
#[cfg(target_os = "windows")]
impl HttpInput {
    pub fn new<'a>(client: reqwest::Client, host: &str) -> Box<Input + 'a> {
        Box::new(HttpInput {
            client,
            host: host.to_owned(),
        })
    }
}
#[cfg(target_os = "windows")]
impl Input for HttpInput {
    fn domains(&self) -> Box<Domains + '_> {
        Box::new(HttpDomains::new(&self.client, &self.host))
    }
    fn device<'a>(&'a self, domain: &'a str, evdev: &'a str) -> Result<Box<Device + '_>, Error> {
        Ok(Box::new(HttpDevice::new(&self.client, &self.host, domain, evdev)))
    }
}

pub trait Domains {
    fn list(&self) -> Result<Vec<String>, Error>;
}

pub struct NativeDomains<'a>(&'a Connection);
impl<'a> NativeDomains<'a> {
    pub fn new(conn: &'a Connection) -> NativeDomains<'a> {
        NativeDomains(conn)
    }
}
impl<'a> Domains for NativeDomains<'a> {
    fn list(&self) -> Result<Vec<String>, Error> {
        self.0.list_all_domains(::virt::connect::VIR_CONNECT_LIST_DOMAINS_ACTIVE)?
            .iter()
            .map(|d| d.get_name())
            .collect::<Result<Vec<String>, ::virt::error::Error>>()
            .map_err(|e| e.into())
    }
}

#[cfg(target_os = "windows")]
pub struct HttpDomains<'a> {
    client: &'a reqwest::Client,
    url: String
}
#[cfg(target_os = "windows")]
impl<'a> HttpDomains<'a> {
    pub fn new(client: &'a reqwest::Client, host: &'a str) -> HttpDomains<'a> {
        HttpDomains {
            client,
            url: format!("{}/domains", host)
        }
    }
}
#[cfg(target_os = "windows")]
impl<'a> Domains for HttpDomains<'a> {
    fn list(&self) -> Result<Vec<String>, Error> {
        self.client
            .get(&self.url)
            .send()
            .map_err(|e| Error::Reqwest(e.to_string()))?
            .json()
            .map_err(|e| Error::Reqwest(e.to_string()))
    }
}

pub trait Device {
    fn evdev(&self) -> &str;
    fn domain(&self) -> &str;

    fn attached(&self) -> bool;

    fn attach(&self) -> Result<(), Error>;
    fn detach(&self) -> Result<(), Error>;
    fn toggle(&self) -> Result<bool, Error> {
        Ok(if self.attached() {
            self.detach()?;
            true
        } else {
            self.attach()?;
            false
        })
    }
}

static mut GLOBAL_CONN: Option<Connection> = None;
pub unsafe fn open_native_global_conn(uri: &str) -> Result<(), ::virt::error::Error> {
    GLOBAL_CONN = Some(Connection::open(uri)?);
    Ok(())
}
pub unsafe fn close_native_global_conn() -> Result<(), Error> {
    match GLOBAL_CONN {
        None => Err(Error::GlobalConnNotOpen),
        Some(ref mut conn) => conn.close().map(|_| ()).map_err(|e| Error::Virt(e))
    }
}
pub fn get_native_global_conn() -> Option<&'static Connection> {
    unsafe {
        match GLOBAL_CONN {
            Some(ref conn) => Some(&conn),
            None => None
        }
    }
}

#[derive(Serialize)]
pub struct NativeDevice {
    evdev: String,
    domain: Domain,

    #[serde(skip)]
    domain_name: String,
    #[serde(skip)]
    xml: String,
}

impl NativeDevice {
    pub fn new(domain: Domain, evdev: String) -> Result<Self, Error> {
        // we only want to check if the evdev exists on the host
        #[cfg(target_os = "linux")]
        {
            let evdev_path = PathBuf::from(evdev.clone());
            if !evdev_path.exists() {
                return Err(Error::BadEvdev(evdev));
            }

            let flags = SFlag::from_bits_truncate(match stat(&evdev_path) {
                Ok(s) => s,
                Err(_) => return Err(Error::BadEvdev(evdev))
            }.st_mode);
            debug!("evdev {:?} st_mode: {:#x}", evdev, flags);
            if !flags.contains(SFlag::S_IFCHR) {
                return Err(Error::BadEvdev(evdev));
            }
        }

        let domain_name = domain.get_name()?;
        let xml = format!(include_str!("attach_detach.xml"), evdev=evdev);
        Ok(NativeDevice {
            evdev,
            domain,
            domain_name,
            xml
        })
    }
}

impl Device for NativeDevice {
    fn evdev(&self) -> &str {
        &self.evdev
    }
    fn domain(&self) -> &str {
        &self.domain_name
    }

    fn attached(&self) -> bool {
        match self.domain.get_xml_desc(VIR_DOMAIN_NONE) {
            Ok(xml) => xml.contains(&self.evdev),
            Err(_) => false
        }
    }

    fn attach(&self) -> Result<(), Error> {
        if self.attached() {
            return Err(Error::BadState("Device already attached!"));
        }

        match self.domain.attach_device_flags(&self.xml, VIR_DOMAIN_AFFECT_LIVE) {
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
    fn detach(&self) -> Result<(), Error> {
        if !self.attached() {
            return Err(Error::BadState("Device not attached!"));
        }

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
impl<'de> Deserialize<'de> for NativeDevice {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field { Evdev, Domain }

        struct DeviceVisitor;
        impl<'de> Visitor<'de> for DeviceVisitor {
            type Value = NativeDevice;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct NativeDevice")
            }

            fn visit_map<V>(self, mut map: V) -> Result<NativeDevice, V::Error>
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

                            domain = match get_native_global_conn() {
                                None => return Err(de::Error::custom(Error::GlobalConnNotOpen)),
                                Some(ref conn) => Some(::virt::domain::Domain::lookup_by_name(&conn, map.next_value()?).map_err(de::Error::custom)?.into())
                            }
                        }
                    }
                }

                let evdev = evdev.ok_or_else(|| de::Error::missing_field("evdev"))?;
                let domain = domain.ok_or_else(|| de::Error::missing_field("domain"))?;
                NativeDevice::new(domain, evdev).map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_struct("NativeDevice", &["evdev", "domain"], DeviceVisitor)
    }
}

#[cfg(target_os = "windows")]
#[derive(Deserialize)]
struct HttpDeviceStatus {
    attached: bool
}

#[cfg(target_os = "windows")]
#[derive(Serialize)]
pub struct HttpDevice<'a> {
    #[serde(skip)]
    client: &'a reqwest::Client,
    #[serde(skip)]
    url: String,

    domain: &'a str,
    evdev: &'a str,
}
#[cfg(target_os = "windows")]
impl<'a> HttpDevice<'a> {
    pub fn new(client: &'a reqwest::Client, host: &'a str, domain: &'a str, evdev: &'a str) -> HttpDevice<'a> {
        HttpDevice {
            client,
            url: format!("{}/device", host),

            domain,
            evdev,
        }
    }
}
#[cfg(target_os = "windows")]
impl<'a> Device for HttpDevice<'a> {
    fn domain(&self) -> &str {
        &self.domain
    }
    fn evdev(&self) -> &str {
        &self.evdev
    }

    fn attached(&self) -> bool {
        match self.client
            .post(&format!("{}/status", self.url))
            .json(self)
            .send() {
            Ok(mut res) => match res.json::<HttpDeviceStatus>() {
                Ok(s) => s.attached,
                Err(_) => false
            },
            Err(_) => false
        }
    }

    fn attach(&self) -> Result<(), Error> {
        if self.attached() {
            warn!("device at '{}' is already attached", self.evdev);
        }

        let mut res = self.client
            .post(&self.url)
            .json(self)
            .send()
            .map_err(|e| Error::Reqwest(e.to_string()))?;
        if !res.status().is_success() {
            return Err(Error::Reqwest(res.text().unwrap_or(String::from("failed to decode response"))));
        }

        Ok(())
    }
    fn detach(&self) -> Result<(), Error> {
        if !self.attached() {
            warn!("device at '{}' is already detached", self.evdev);
        }

        let mut res = self.client
            .delete(&self.url)
            .json(self)
            .send()
            .map_err(|e| Error::Reqwest(e.to_string()))?;
        if !res.status().is_success() {
            return Err(Error::Reqwest(res.text().unwrap_or(String::from("failed to decode response"))));
        }

        Ok(())
    }
}
