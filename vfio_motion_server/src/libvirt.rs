// Some structs imported from libvirt are only pointer.
#![allow(improper_ctypes)]

// We don't want rustc to warn on this because it's imported from
// libvirt.
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

macro_rules! c_chars_to_string {
    ($x:expr) => {{
        let ret = ::std::ffi::CStr::from_ptr($x).to_string_lossy().into_owned();
        libc::free($x as *mut libc::c_void);
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

extern crate libc;
extern crate virt;

use virt::error::Error;
use virt::domain::sys::{virDomainPtr};

pub struct Connection(virt::connect::Connect);
impl Drop for Connection {
    fn drop(&mut self) {
        self.close().unwrap();
    }
}
impl Deref for Connection {
    type Target = virt::connect::Connect;
    fn deref(&self) -> &virt::connect::Connect {
        &self.0
    }
}
impl DerefMut for Connection {
    fn deref_mut(&mut self) -> &mut virt::connect::Connect {
        &mut self.0
    }
}

impl Connection {
    pub fn open(uri: &str) -> Result<Connection, virt::error::Error> {
        Ok(Connection(virt::connect::Connect::open(uri)?))
    }
}

pub struct Domain<'a>(&'a virt::domain::Domain);
impl<'a> Deref for Domain<'a> {
    type Target = virt::domain::Domain;
    fn deref(&self) -> &virt::domain::Domain {
        &self.0
    }
}

impl<'a> From<&'a virt::domain::Domain> for Domain<'a> {
    fn from(d: &'a virt::domain::Domain) -> Self {
        Domain(d)
    }
}
#[link(name = "virt-qemu")]
extern "C" {
    fn virDomainQemuMonitorCommand(ptr: virDomainPtr, cmd: *const libc::c_char, result: *mut *mut libc::c_char, flags: libc::c_uint) -> libc::c_int;
}
impl<'a> Domain<'a> {
    pub fn qemu_monitor_command(&self, command: &str, flags: u32) -> Result<Option<String>, Error> {
        unsafe {
            let mut result = ptr::null_mut();
            let ret = virDomainQemuMonitorCommand(self.0.as_ptr(), string_to_c_chars!(command), &mut result, flags);
            if ret == -1 {
                return Err(Error::new());
            }

            if result.is_null() {
                Ok(None)
            } else {
                Ok(Some(c_chars_to_string!(result)))
            }
        }
    }
}
