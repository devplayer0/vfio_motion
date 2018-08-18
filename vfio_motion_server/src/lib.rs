extern crate config;

use std::error::Error;

use config::Config;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
