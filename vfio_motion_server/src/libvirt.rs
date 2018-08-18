use std::ops::{Deref, DerefMut};

extern crate virt;

pub struct Connection {
    conn: virt::connect::Connect,
}
impl Drop for Connection {
    fn drop(&mut self) {
        self.close().unwrap();
    }
}
impl Deref for Connection {
    type Target = virt::connect::Connect;
    fn deref(&self) -> &virt::connect::Connect {
        &self.conn
    }
}
impl DerefMut for Connection {
    fn deref_mut(&mut self) -> &mut virt::connect::Connect {
        &mut self.conn
    }
}

impl Connection {
    pub fn open(uri: &str) -> Result<Connection, virt::error::Error> {
        Ok(Connection {
            conn:virt::connect::Connect::open(uri)? 
        })
    }
}
