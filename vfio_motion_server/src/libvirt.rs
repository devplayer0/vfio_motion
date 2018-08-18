extern crate virt;

pub struct Connection {
    conn: virt::connect::Connect,
}
impl Connection {
    pub fn open(uri: &str) -> Result<Connection, virt::error::Error> {
        Ok(Connection {
            conn:virt::connect::Connect::open(uri)? 
        })
    }
    pub fn get(&self) -> &virt::connect::Connect {
        &self.conn
    }
}
impl Drop for Connection {
    fn drop(&mut self) {
        self.conn.close().unwrap();
    }
}
