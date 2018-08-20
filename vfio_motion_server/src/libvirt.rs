// Some structs imported from libvirt are only pointer.
#![allow(improper_ctypes)]

// We don't want rustc to warn on this because it's imported from
// libvirt.
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

macro_rules! c_chars_to_string {
    ($x:expr) => {{
        let ret = ::std::ffi::CStr::from_ptr($x).to_string_lossy().into_owned();
        ::libc::free($x as *mut c_void);
        ret
    }};

    ($x:expr, nofree) => {{
        ::std::ffi::CStr::from_ptr($x).to_string_lossy().into_owned()
    }};

}
macro_rules! string_to_c_chars {
    ($x:expr) => (::std::ffi::CString::new($x).unwrap().as_ptr())
}

use std::ops::{Deref, DerefMut};
use std::ptr;
use std::mem;
use std::fmt;

use ::virt::domain::sys::virDomainPtr;
use ::libc::{c_uint, c_int, c_char, c_void};
use ::serde::ser::{self, Serialize, Serializer};
use ::serde::de::{self, Deserialize, Deserializer, Visitor};

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        SerdeError(err: ::serde_json::Error) {
            from()
            description(err.description())
        }
        Virt(err: ::virt::error::Error) {
            from()
            description(err.description())
        }
        QemuMonitor(msg: ::serde_json::Value) {
            display("Qemu Monitor command error: {:#?}", msg)
        }
    }
}

pub struct Connection(::virt::connect::Connect);
impl Drop for Connection {
    fn drop(&mut self) {
        trace!("closing qemu connection");
        self.close().unwrap();
    }
}
impl Deref for Connection {
    type Target = ::virt::connect::Connect;
    fn deref(&self) -> &::virt::connect::Connect {
        &self.0
    }
}
impl DerefMut for Connection {
    fn deref_mut(&mut self) -> &mut ::virt::connect::Connect {
        &mut self.0
    }
}

impl Connection {
    pub fn open(uri: &str) -> Result<Connection, ::virt::error::Error> {
        Ok(Connection(::virt::connect::Connect::open(uri)?))
    }
}

pub type QemuMonitorCommandFlags = c_uint;
pub const VIR_DOMAIN_QEMU_MONITOR_COMMAND_DEFAULT: QemuMonitorCommandFlags = 0;
pub const VIR_DOMAIN_QEMU_MONITOR_COMMAND_HMP: QemuMonitorCommandFlags = 1;

pub type virErrorFunc = unsafe extern "C" fn(*mut c_void, ::virt::error::sys::virErrorPtr);
#[link(name = "virt-qemu")]
extern "C" {
    fn virSetErrorFunc(ctx: *mut c_void, handler: virErrorFunc);
    fn virDomainQemuMonitorCommand(ptr: virDomainPtr, cmd: *const c_char, result: *mut *mut c_char, flags: c_uint) -> c_int;
}

pub type VirtErrorHandler<T> = fn(Box<Option<T>>, ::virt::error::Error);
struct VirtErrorData<T> {
    handler: VirtErrorHandler<T>,
    ctx: Box<Option<T>>
}
unsafe extern "C" fn _error_handler<T>(_ctx: *mut c_void, _err: ::virt::error::sys::virErrorPtr) {
    let _ctx: Box<VirtErrorData<T>> = mem::transmute(_ctx);
    let err = ::virt::error::Error {
        code: (*_err).code,
        domain: (*_err).domain,
        message: c_chars_to_string!((*_err).message, nofree),
        level: ::virt::error::ErrorLevel::from((*_err).level)
    };

    (_ctx.handler)(_ctx.ctx, err);
}
pub fn set_error_handler<T>(ctx: Box<Option<T>>, handler: VirtErrorHandler<T>) {
    unsafe {
        let ctx = Box::new(VirtErrorData {
            handler,
            ctx
        });

        virSetErrorFunc(mem::transmute(ctx), _error_handler::<T>);
    }
}

pub struct Domain(::virt::domain::Domain);
impl Deref for Domain {
    type Target = ::virt::domain::Domain;
    fn deref(&self) -> &::virt::domain::Domain {
        &self.0
    }
}

impl From<::virt::domain::Domain> for Domain {
    fn from(d: ::virt::domain::Domain) -> Self {
        Domain(d)
    }
}
impl Domain {
    pub fn qemu_monitor_command(&self, command: &str, flags: QemuMonitorCommandFlags) -> Result<Option<::serde_json::Value>, Error> {
        unsafe {
            let mut result = ptr::null_mut();
            let ret = virDomainQemuMonitorCommand(self.0.as_ptr(), string_to_c_chars!(command), &mut result, flags);
            trace!("qemu monitor ret: {}", ret);

            if ret != 0 {
                return Err(::virt::error::Error::new().into());
            }

            Ok(if !result.is_null() {
                let msg = c_chars_to_string!(result);
                let parse: Result<::serde_json::Value, ::serde_json::Error> = ::serde_json::from_str(&msg);
                Some(if let Ok(val) = parse {
                    if val.as_object().unwrap().contains_key("error") {
                        return Err(Error::QemuMonitor(val.as_object().unwrap()["error"].clone()));
                    } else {
                        val
                    }
                } else {
                    json!(msg)
                })
            } else {
               None
            })
        }
    }
}
impl Serialize for Domain {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let name = self.get_name().map_err(ser::Error::custom)?;
        serializer.serialize_str(&name)
    }
}
impl<'de> Deserialize<'de> for Domain {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DomainVisitor;
        impl<'de> Visitor<'de> for DomainVisitor {
            type Value = Domain;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Domain")
            }

            fn visit_str<E>(self, value: &str) -> Result<Domain, E>
            where
                E: de::Error,
            {
                let conn = ::libvirt::Connection::open("qemu:///system").map_err(de::Error::custom)?;
                Ok(::virt::domain::Domain::lookup_by_name(&conn, value).map_err(de::Error::custom)?.into())
            }
        }

        deserializer.deserialize_str(DomainVisitor)
    }
}
