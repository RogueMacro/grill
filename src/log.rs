use std::{cell::RefCell, io::BufWriter};

use indicatif::ProgressBar;

thread_local! {
    static PROGRESS_BAR: RefCell<Option<ProgressBar>> = RefCell::new(None);
}

struct Logger {
    logger: env_logger::Logger,
}

impl Logger {
    pub fn new() -> Self {
        let buf = BufWriter::new(Vec::new());
        let logger = env_logger::builder().target(Target::Pipe(buf)).init();
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        self.logger.enabled(metadata)
    }

    fn log(&self, record: &log::Record) {
        self.logger.log(record)
    }

    fn flush(&self) {
        self.logger.flush()
    }
}
