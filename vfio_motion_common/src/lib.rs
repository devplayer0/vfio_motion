#[macro_use]
extern crate quick_error;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

extern crate config;
extern crate serde;
extern crate virt;
#[cfg(target_os = "linux")]
extern crate nix;
extern crate libc;
#[cfg(target_os = "windows")]
extern crate reqwest;

pub mod util;
pub mod libvirt;
pub mod input;
