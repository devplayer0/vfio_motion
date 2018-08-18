use std::error::Error;

extern crate config;
extern crate virt;

use config::Config;

mod libvirt;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let uri: String = config.get("libvirt_uri")?;
    let conn = libvirt::Connection::open(&uri)?;

    let domains = conn.get().list_all_domains(virt::connect::VIR_CONNECT_LIST_DOMAINS_ACTIVE)?;
    for domain in domains {
        println!("libvirt domain: {}", domain.get_name()?);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
