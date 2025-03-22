use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};

pub struct NesLogger;

static LOGGER: NesLogger = NesLogger;

impl log::Log for NesLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Debug
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let level_str = match record.level() {
                Level::Error => "\x1b[31mERROR\x1b[0m", // Red
                Level::Warn => "\x1b[33mWARN\x1b[0m",   // Yellow
                Level::Info => "\x1b[32mINFO\x1b[0m",   // Green
                Level::Debug => "\x1b[36mDEBUG\x1b[0m", // Cyan
                Level::Trace => "\x1b[90mTRACE\x1b[0m", // Grey
            };
            println!(
                "[{}] {}: {}",
                level_str,
                record.target(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

pub fn init() -> Result<(), SetLoggerError> {
    // Set the global logger
    log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Debug))
}
