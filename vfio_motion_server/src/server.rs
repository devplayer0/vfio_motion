use ::rocket::config::Config;
use ::rocket::error::{LaunchError};
use ::rocket::http::Status;
use ::rocket::response::status;
use ::rocket_contrib::{SerdeError, Json};

use ::input::{self, Device, NativeDevice, Domains, NativeDomains};

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

#[post("/device/status", data="<device>")]
fn attached(device: Result<Json<NativeDevice>, SerdeError>) -> Result<Json, status::Custom<Json<ErrorMsg>>> {
    match device {
        Ok(Json(d)) => {
            debug!("handling status of evdev at '{:?}'", d.evdev());
            Ok(Json(json!({ "attached": d.attached() })))
        },
        Err(e) => Err(ErrorMsg::serde(e))
    }
}
#[post("/device", data="<device>")]
fn attach(device: Result<Json<NativeDevice>, SerdeError>) -> Result<status::NoContent, status::Custom<Json<ErrorMsg>>> {
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
#[delete("/device", data="<device>")]
fn detach(device: Result<Json<NativeDevice>, SerdeError>) -> Result<status::NoContent, status::Custom<Json<ErrorMsg>>> {
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

#[get("/domains")]
fn domains() -> Result<Json, status::Custom<Json<ErrorMsg>>> {
    match NativeDomains::new(input::get_native_global_conn().unwrap()).list() {
        Ok(doms) => Ok(Json(json!(doms))),
        Err(e) => Err(ErrorMsg::input(e))
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
        .mount("/", routes![attached, attach, detach, domains])
        .catch(catchers![not_found, internal_error])
        .launch()
}
