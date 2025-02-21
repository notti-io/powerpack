//! A simple file logger for use in Alfred workflows.
//!
//! The logger comes preconfigured with sensible defaults for use in Alfred
//! workflows.
//!
//! # Usage
//!
//! This crate is re-exported from the `powerpack` crate, so you can access it
//! using `powerpack::logger`.
//!
//! Using all the defaults. The following will write to a file at
//! ```text
//! {alfred_workflow_cache}/powerpack.log
//! ```
//!
//! And the log level will be set to `Debug` if the workflow is running in debug
//! mode, otherwise it will be set to `Info`.
//!
//! ```no_run
//! # mod powerpack { pub extern crate powerpack_logger as logger; } // mock re-export
//! use powerpack::logger;
//!
//! logger::Builder::new().init();
//! ```
//!
//! `Logger::builder()` returns a builder where you can further configure the
//! logger. For example to set the filename and the log level.
//!
//! ```no_run
//! # mod powerpack { pub extern crate powerpack_logger as logger; } // mock re-export
//! use powerpack::logger;
//!
//! const FILENAME: &str = concat!(env!("CARGO_PKG_NAME"), "-", env!("CARGO_PKG_VERSION"), ".log");
//! logger::Builder::new()
//!     .filename(FILENAME)
//!     .max_level(logger::LevelFilter::Warn)
//!     .init();
//! ```

use std::borrow::Cow;
use std::fs;
use std::io::prelude::*;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use log::Log;
use thiserror::Error;

use powerpack_env as env;

pub use log::LevelFilter;

/// An error that can occur when using the logger.
#[derive(Debug, Error)]
pub enum Error {
    /// Raised when the home directory cannot be determined.
    #[error("home directory not found")]
    NoHomeDir,

    /// An I/O error occurred.
    #[error("io error")]
    Io(#[from] std::io::Error),

    /// Failed to set the logger.
    #[error("failed to set logger")]
    SetLogger(#[from] log::SetLoggerError),
}

/// A simple logger that writes log messages to a file.
#[derive(Debug, Clone)]
pub struct Logger {
    file: Arc<Mutex<fs::File>>,
    max_level: LevelFilter,
}

/// A builder for configuring the file logger.
#[derive(Debug, Clone)]
pub struct Builder {
    directory: Option<PathBuf>,
    filename: Option<Cow<'static, str>>,
    max_level: Option<LevelFilter>,
}

impl Default for Builder {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
    /// Returns a new builder for configuring the logger.
    #[inline]
    pub fn new() -> Self {
        Self {
            directory: None,
            filename: None,
            max_level: None,
        }
    }

    /// Set the directory where the log file will be stored.
    ///
    /// Defaults to the Alfred workflow cache directory.
    #[inline]
    pub fn directory(mut self, dir: impl Into<PathBuf>) -> Self {
        self.directory = Some(dir.into());
        self
    }

    /// Set the name of the log file relative to the directory.
    ///
    /// Defaults to `powerpack.log`.
    #[inline]
    pub fn filename(mut self, filename: impl Into<Cow<'static, str>>) -> Self {
        self.filename = Some(filename.into());
        self
    }

    /// Set the maximum log level
    #[inline]
    pub fn max_level(mut self, max_level: LevelFilter) -> Self {
        self.max_level = Some(max_level);
        self
    }

    fn build(self) -> Result<Logger, Error> {
        let directory = match self.directory {
            Some(directory) => directory,
            None => env::try_workflow_cache_or_default().ok_or(Error::NoHomeDir)?,
        };
        let filename = self.filename.as_deref().unwrap_or("powerpack.log");
        let path = directory.join(filename);

        fs::create_dir_all(directory)?;

        let file = Arc::new(Mutex::new(
            fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)?,
        ));

        let max_level = self.max_level.unwrap_or_else(|| {
            if env::is_debug() {
                LevelFilter::Debug
            } else {
                LevelFilter::Info
            }
        });

        Ok(Logger { file, max_level })
    }

    /// Try to initialize the logger
    #[inline]
    pub fn try_init(self) -> Result<(), Error> {
        let logger = self.build()?;
        let max_level = logger.max_level;
        log::set_boxed_logger(Box::new(logger))?;
        log::set_max_level(max_level);
        Ok(())
    }

    /// Initialize the logger
    ///
    /// # Panics
    ///
    /// Panics if the logger has already been initialized or there are IO errors
    /// when creating/accessing the log file.
    ///
    /// Use [`Builder::try_init`] if you want to handle the error.
    #[track_caller]
    #[inline]
    pub fn init(self) {
        self.try_init().expect("failed to initialize logger");
    }

    /// Initialize the logger if it hasn't already been initialized.
    ///
    /// # Panics
    ///
    /// Panics if there are IO errors when creating/accessing the log file.
    #[track_caller]
    #[inline]
    pub fn init_idempotent(self) {
        match self.try_init() {
            Ok(()) => {}
            Err(Error::SetLogger(_)) => {}
            r => r.expect("failed to initialize logger"),
        }
    }
}

impl Logger {
    fn try_log(&self, record: &log::Record) -> Result<(), Box<dyn std::error::Error + '_>> {
        let time = jiff::Timestamp::now().strftime("%Y-%m-%dT%H:%M:%S");
        let mut f = self.file.lock()?;
        writeln!(f, "[{}] [{}] {}", time, record.level(), record.args())?;
        f.flush()?;
        Ok(())
    }

    fn try_flush(&self) -> Result<(), Box<dyn std::error::Error + '_>> {
        let mut f = self.file.lock()?;
        f.flush()?;
        Ok(())
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.max_level
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            self.try_log(record).expect("failed to log message");
        }
    }

    fn flush(&self) {
        self.try_flush().expect("failed to flush log file");
    }
}
