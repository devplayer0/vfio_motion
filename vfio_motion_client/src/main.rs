extern crate vfio_motion_common;

use ::vfio_motion_common::libvirt::{self, Connection};

fn main() {
    let conn = libvirt::Connection::open("qemu+tcp://10.0.122.1/system").unwrap();
    for domain in conn.list_all_domains(1).unwrap() {
        println!("got domain: {}", domain.get_name().unwrap());
    }
}
