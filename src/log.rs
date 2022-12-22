use std::fs::File;

use anyhow::Result;
use indicatif::MultiProgress;
use lazy_static::lazy_static;
use log::LevelFilter;

use crate::paths;

lazy_static! {
    static ref PROGRESS_BAR: MultiProgress = MultiProgress::new();
}

pub fn init(max_level: LevelFilter) -> Result<()> {
    let console_logger = Box::new(ConsoleLogger::new(max_level));
    let file_logger = simplelog::WriteLogger::new(
        log::LevelFilter::max(),
        Default::default(),
        File::create(paths::home().join("log.txt"))?,
    );
    multi_log::MultiLogger::init(vec![console_logger, file_logger], log::Level::max())
        .map_err(anyhow::Error::from)
}

pub fn get_multi_progress() -> &'static MultiProgress {
    &PROGRESS_BAR
}

struct ConsoleLogger {
    inner: env_logger::Logger,
}

impl ConsoleLogger {
    pub fn new(max_level: LevelFilter) -> Self {
        Self {
            inner: env_logger::builder()
                .format_timestamp(None)
                .format_target(false)
                .filter_level(max_level)
                .build(),
        }
    }
}

impl log::Log for ConsoleLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        self.inner.enabled(metadata)
    }

    fn log(&self, record: &log::Record) {
        if self.inner.matches(record) {
            get_multi_progress().suspend(|| {
                self.inner.log(record);
            });
        }
    }

    fn flush(&self) {
        self.inner.flush();
    }
}
