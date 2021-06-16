use log::{Metadata, Record, LevelFilter};

struct Logger;
impl log::Log for Logger {
    fn enabled(&self, _: &Metadata) -> bool { true }
    fn log(&self, record: &Record) { println!("{}", record.args()); }
    fn flush(&self) {}
}

static LOGGER: Logger = Logger;

pub fn init(quiet: bool) {
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(if quiet { LevelFilter::Error } else { LevelFilter::Info });
}
