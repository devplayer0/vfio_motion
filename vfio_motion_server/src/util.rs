use ::log::LevelFilter;
use ::rocket::config::LoggingLevel;

pub fn rocket_log_level(level: LevelFilter) -> LoggingLevel {
    match level {
        LevelFilter::Off | LevelFilter::Error | LevelFilter::Warn => LoggingLevel::Critical,
        LevelFilter::Info => LoggingLevel::Normal,
        LevelFilter::Debug | LevelFilter::Trace => LoggingLevel::Debug,
    }
}
