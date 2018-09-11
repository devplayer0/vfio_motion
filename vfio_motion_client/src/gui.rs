use std::error::Error;

use ::gtk;
use gtk::prelude::*;

use ::config::Config;

const GLADE_SRC: &'static str = include_str!("ui.glade");

pub fn run(_config: Config) -> Result<(), Box<dyn Error>> {
    gtk::init()?;

    let builder = gtk::Builder::new_from_string(GLADE_SRC);

    let window: gtk::Window = builder.get_object("window").unwrap();
    window.show_all();

    gtk::main();

    Ok(())
}
