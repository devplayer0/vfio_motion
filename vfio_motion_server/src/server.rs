use ::rocket::config::Config;
use ::rocket::error::{LaunchError};
use ::rocket::http::Status;
use ::rocket::response::status;
use ::rocket_contrib::{SerdeError, Json};

use input::{self, Device};

macro_rules! error_msg {
    ($name:ident, $status:ident, $err:ty) => (
        pub fn $name(err: $err) -> status::Custom<Json<ErrorMsg>> {
            status::Custom(Status::$status, Json(ErrorMsg {
                message: format!("{}", err)
            }))
        }
    )
}

#[derive(Debug, Serialize)]
pub struct ErrorMsg {
    message: String
}
impl ErrorMsg {
    error_msg!(serde, BadRequest, SerdeError);
    error_msg!(input, InternalServerError, input::Error);
}

#[post("/", data="<device>")]
fn attach(device: Result<Json<Device>, SerdeError>) -> Result<status::NoContent, status::Custom<Json<ErrorMsg>>> {
    match device {
        Ok(Json(d)) => {
            debug!("handling attach of evdev at '{:?}'", d.evdev());
            match d.attach() {
                Ok(()) => Ok(status::NoContent),
                Err(e) => Err(ErrorMsg::input(e))
            }
        },
        Err(e) => Err(ErrorMsg::serde(e))
    }
}
#[delete("/", data="<device>")]
fn detach(device: Result<Json<Device>, SerdeError>) -> Result<status::NoContent, status::Custom<Json<ErrorMsg>>> {
    match device {
        Ok(Json(d)) => {
            debug!("handling detach of evdev at '{:?}'", d.evdev());
            match d.detach() {
                Ok(()) => Ok(status::NoContent),
                Err(e) => Err(ErrorMsg::input(e))
            }
        },
        Err(e) => Err(ErrorMsg::serde(e))
    }
}

#[catch(404)]
fn not_found() -> Json {
    Json(json!({ "message": "not found" }))
}
#[catch(500)]
fn internal_error() -> Json {
    Json(json!({ "message": "internal server error" }))
}

pub fn run(config: Config) -> LaunchError {
    // Unfortunately since were using the same log framework as Rocket, log to false has no effect
    ::rocket::custom(config, ::log::max_level() >= ::log::LevelFilter::Debug)
        .mount("/", routes![attach, detach])
        .catch(catchers![not_found, internal_error])
        .launch()
}
