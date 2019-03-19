//! A log thread and thread-local log proxy combination.
//!
//! This module provides logging functionality to run a dedicated log thread in
//! combination with one or more thread-local log proxy instances. The log
//! thread provides the endpoint used by the log proxy instances to send their
//! log records. Log proxy instances run in different threads or child
//! processes.
//!
//! # Usage
//!
//! Start by spawning a [`LogThread`] from the main thread. Next, initialize a
//! [`LogProxy`] instance per thread or child process. A log [`Record`] can be
//! generated using the provided [`macros`]. The thread-local [`LogProxy`]
//! forwards the records to the [`LogThread`] for logging.
//!
//! Both [`LogThread`] and [`LogProxy`] filter [`Records`] based on their
//! [`Loglevel`]. The behavior is defined by setting a [`LoglevelFilter`].
//!
//! # Basic Example
//!
//! ```rust
//! use dqcsim::{
//!     log::{init, proxy::LogProxy, thread::LogThread, LoglevelFilter},
//!     note,
//! };
//!
//! // Spawn the log thread. This also starts a thread-local log proxy in the main
//! // thread.
//! let log_thread = LogThread::spawn(
//!     "main_thread",
//!     LoglevelFilter::Note,
//!     LoglevelFilter::Note,
//!     None,
//! )
//! .unwrap();
//!
//! // Grab a copy of the log thread sender to use in the log proxy.
//! let log_endpoint = log_thread.get_sender().unwrap();
//!
//! // Spawn an other thread.
//! std::thread::spawn(move || {
//!     // Construct a log proxy instance which connects to the log thread endpoint.
//!     let log_proxy = LogProxy::boxed("other_thread", log_endpoint);
//!
//!     // Initialize the thread-local logger to enable forwarding of log records to
//!     // the log thread.
//!     init(log_proxy, LoglevelFilter::Trace);
//!
//!     // Generate a log record
//!     note!("Note from thread via proxy");
//! })
//! .join();
//!
//! // This log records is also forwarded to the log thread by the log proxy running
//! // in the main thread.
//! note!("Note from main thread via proxy started by log_thread spawn function");
//!
//! ```
//!
//! # Inspired by
//! * [`log`]
//! * sfackler's [comment](https://github.com/rust-lang-nursery/log/issues/57#issuecomment-143383896)
//!
//! [`LogThread`]: ./thread/struct.LogThread.html
//! [`LogProxy`]: ./proxy/struct.LogProxy.html
//! [`Record`]: ./struct.Record.html
//! [`Records`]: ./struct.Record.html
//! [`Loglevel`]: ./enum.Loglevel.html
//! [`LoglevelFilter`]: ./enum.LoglevelFilter.html
//! [`macros`]: #macros
//! [`log`]: https://github.com/rust-lang-nursery/log

// This re-export is required as the trait needs to be in scope in the log
// macro.
#[doc(hidden)]
pub use ref_thread_local as _ref_thread_local;

pub mod channel;
pub mod proxy;
pub mod router;
pub mod stdio;
pub mod tee_file;
pub mod thread;

use crate::{
    error,
    error::{ErrorKind, ResultExt},
    log::channel::Sender,
};
use enum_variants::EnumVariants;
use lazy_static::lazy_static;
use ref_thread_local::ref_thread_local;
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, fmt};

/// The Log trait.
///
/// # Implementing the Log trait.
///
/// ```rust
/// use dqcsim::{log};
/// ```
pub trait Log {
    /// Returns the name of this logger
    fn name(&self) -> &str;
    /// Log the incoming record
    fn log(&self, record: Record);
}

thread_local! {
    /// The thread-local logger.
    static LOGGER: RefCell<Option<Box<dyn Log>>> = RefCell::new(None);
    /// The thread-local maximum log level. Defaults to off.
    #[doc(hidden)]
    pub static LOGLEVEL: RefCell<LoglevelFilter> = RefCell::new(LoglevelFilter::Off);
}

lazy_static! {
    // Cache the process id.
    #[doc(hidden)]
    pub static ref PID: u32 = std::process::id();
}

ref_thread_local! {
    // Cache the thread id.
    #[doc(hidden)]
    // Don't ask. (rust-lang/rust #52780)
    pub static managed TID: u64 = unsafe { std::mem::transmute(std::thread::current().id()) };
}

/// Loglevel for log records.
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Serialize, Deserialize, EnumVariants)]
pub enum Loglevel {
    /// This loglevel is to be used for reporting a fatal error, resulting from
    /// the owner of the logger getting into an illegal state from which it
    /// cannot recover. Such problems are also reported to the API caller via
    /// Result::Err if applicable.
    Fatal = 1,

    /// This loglevel is to be used for reporting or propagating a non-fatal
    /// error caused by the API caller doing something wrong. Such problems are
    /// also reported to the API caller via Result::Err if applicable.
    Error,

    /// This loglevel is to be used for reporting that a called API/function is
    /// telling us we did something wrong (that we weren't expecting), but we
    /// can recover. For instance, for a failed connection attempt to something
    /// that really should not be failing, we can still retry (and eventually
    /// report critical or error if a retry counter overflows). Since we're
    /// still trying to rectify things at this point, such problems are NOT
    /// reported to the API/function caller via Result::Err.
    Warn,

    /// This loglevel is to be used for reporting information specifically
    /// requested by the user/API caller, such as the result of an API function
    /// requested through the command line, or an explicitly captured
    /// stdout/stderr stream.
    Note,

    /// This loglevel is to be used for reporting information NOT specifically
    /// requested by the user/API caller, such as a plugin starting up or
    /// shutting down.
    Info,

    /// This loglevel is to be used for reporting debugging information useful
    /// for debugging the user of the API provided by the logged instance.
    Debug,

    /// This loglevel is to be used for reporting debugging information useful
    /// for debugging the internals of the logged instance. Such messages would
    /// normally only be generated by debug builds, to prevent them from
    /// impacting performance under normal circumstances.
    Trace,
}

/// LoglevelFilter for implementors of the Log trait.
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Serialize, Deserialize, EnumVariants)]
pub enum LoglevelFilter {
    /// A level lower than all log levels.
    Off = 0,
    /// Corresponds to the `Fatal` log level.
    Fatal,
    /// Corresponds to the `Error` log level.
    Error,
    /// Corresponds to the `Warn` log level.
    Warn,
    /// Corresponds to the `Note` log level.
    Note,
    /// Corresponds to the `Info` log level.
    Info,
    /// Corresponds to the `Debug` log level.
    Debug,
    /// Corresponds to the `Trace` log level.
    Trace,
}

impl Loglevel {
    /// Attempt to convert a LoglevelFilter to a Loglevel.
    ///
    /// Until std::convert::TryFrom is stable. (rust-lang/rust #33417)
    pub fn try_from(levelfilter: LoglevelFilter) -> Result<Loglevel, ()> {
        match levelfilter {
            LoglevelFilter::Fatal => Ok(Loglevel::Fatal),
            LoglevelFilter::Error => Ok(Loglevel::Error),
            LoglevelFilter::Warn => Ok(Loglevel::Warn),
            LoglevelFilter::Note => Ok(Loglevel::Note),
            LoglevelFilter::Info => Ok(Loglevel::Info),
            LoglevelFilter::Debug => Ok(Loglevel::Debug),
            LoglevelFilter::Trace => Ok(Loglevel::Trace),
            LoglevelFilter::Off => Err(()),
        }
    }
}

impl From<Loglevel> for LoglevelFilter {
    fn from(level: Loglevel) -> LoglevelFilter {
        match level {
            Loglevel::Fatal => LoglevelFilter::Fatal,
            Loglevel::Error => LoglevelFilter::Error,
            Loglevel::Warn => LoglevelFilter::Warn,
            Loglevel::Note => LoglevelFilter::Note,
            Loglevel::Info => LoglevelFilter::Info,
            Loglevel::Debug => LoglevelFilter::Debug,
            Loglevel::Trace => LoglevelFilter::Trace,
        }
    }
}

/// Log record metadata.
///
/// The log metadata attached to a [`Record`].
///
/// [`Record`]: ./struct.Record.html
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Metadata {
    /// Loglevel of the log record.
    level: Loglevel,
    module_path: Option<String>,
    file: Option<String>,
    line: Option<u32>,
    timestamp: std::time::SystemTime,
    process: u32,
    thread: u64,
}

/// A log record.
///
/// A log record consists of some metadata and a payload.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Record {
    payload: String,
    metadata: Metadata,
    logger: String,
}

impl Record {
    pub fn payload(&self) -> &str {
        &self.payload
    }
    pub fn level(&self) -> Loglevel {
        self.metadata.level
    }
    pub fn module_path(&self) -> Option<&str> {
        self.metadata.module_path.as_ref().map(String::as_str)
    }
    pub fn file(&self) -> Option<&str> {
        self.metadata.file.as_ref().map(String::as_str)
    }
    pub fn line(&self) -> Option<u32> {
        self.metadata.line
    }
    pub fn timestamp(&self) -> std::time::SystemTime {
        self.metadata.timestamp
    }
    pub fn process(&self) -> u32 {
        self.metadata.process
    }
    pub fn thread(&self) -> u64 {
        self.metadata.thread
    }
    pub fn logger(&self) -> &str {
        self.logger.as_str()
    }
}

impl Record {
    pub fn log(
        payload: impl Into<String>,
        level: Loglevel,
        module_path: impl Into<String>,
        file: impl Into<String>,
        line: u32,
        process: u32,
        thread: u64,
    ) {
        LOGGER.with(|logger| {
            if let Some(ref logger) = *logger.borrow() {
                let record = Record {
                    payload: payload.into(),
                    metadata: Metadata {
                        level,
                        module_path: Some(module_path.into()),
                        file: Some(file.into()),
                        line: Some(line),
                        timestamp: std::time::SystemTime::now(),
                        process,
                        thread,
                    },
                    logger: logger.name().to_owned(),
                };
                logger.log(record);
            }
        });
    }
}

impl fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.payload)
    }
}

/// Update the thread-local logger.
fn update(log: Option<Box<dyn Log>>, level: Option<LoglevelFilter>) -> error::Result<()> {
    LOGGER.with(|logger| {
        let mut logger = logger.try_borrow_mut().context(ErrorKind::LogError(
            "failed to update thread-local log level".to_string(),
        ))?;
        *logger = log;
        LOGLEVEL.with(|loglevel| {
            let mut loglevel = loglevel.try_borrow_mut().context(ErrorKind::LogError(
                "failed to update thread-local log level".to_string(),
            ))?;
            *loglevel = level.unwrap_or(LoglevelFilter::Off);
            Ok(())
        })
    })
}

/// Initialize the thread-local logger.
pub fn init(log: Box<dyn Log>, level: LoglevelFilter) -> error::Result<()> {
    update(Some(log), Some(level))
}

/// Deinitialize the thread-local logger.
pub fn deinit() -> error::Result<()> {
    update(None, None)
}

#[macro_export]
macro_rules! log {
    (target: $target:expr, $lvl:expr, $($arg:tt)+) => ({
        use $crate::log::_ref_thread_local::RefThreadLocal;
        $crate::log::LOGLEVEL.with(|loglevel| {
            if $crate::log::LoglevelFilter::from($lvl) <= *loglevel.borrow() {
                $crate::log::Record::log(
                    format!($($arg)+),
                    $lvl,
                    $target,
                    file!(),
                    line!(),
                    *$crate::log::PID,
                    *$crate::log::TID.borrow()
                );
            }
        });
    });
    ($lvl:expr, $($arg:tt)+) => ($crate::log!(target: module_path!(), $lvl, $($arg)+))
}

#[macro_export]
macro_rules! fatal {
    (target: $target:expr, $($arg:tt)+) => (
        $crate::log!(target: $target, $crate::log::Loglevel::Fatal, $($arg)+);
    );
    ($($arg:tt)+) => (
        $crate::log!($crate::log::Loglevel::Fatal, $($arg)+);
    )
}

#[macro_export]
macro_rules! error {
    (target: $target:expr, $($arg:tt)+) => (
        $crate::log!(target: $target, $crate::log::Loglevel::Error, $($arg)+);
    );
    ($($arg:tt)+) => (
        $crate::log!($crate::log::Loglevel::Error, $($arg)+);
    )
}

#[macro_export]
macro_rules! warn {
    (target: $target:expr, $($arg:tt)+) => (
        $crate::log!(target: $target, $crate::log::Loglevel::Warn, $($arg)+);
    );
    ($($arg:tt)+) => (
        $crate::log!($crate::log::Loglevel::Warn, $($arg)+);
    )
}

#[macro_export]
macro_rules! note {
    (target: $target:expr, $($arg:tt)+) => (
        $crate::log!(target: $target, $crate::log::Loglevel::Note, $($arg)+);
    );
    ($($arg:tt)+) => (
        $crate::log!($crate::log::Loglevel::Note, $($arg)+);
    )
}

#[macro_export]
macro_rules! info {
    (target: $target:expr, $($arg:tt)+) => (
        $crate::log!(target: $target, $crate::log::Loglevel::Info, $($arg)+);
    );
    ($($arg:tt)+) => (
        $crate::log!($crate::log::Loglevel::Info, $($arg)+);
    )
}

#[macro_export]
macro_rules! debug {
    (target: $target:expr, $($arg:tt)+) => (
        $crate::log!(target: $target, $crate::log::Loglevel::Debug, $($arg)+);
    );
    ($($arg:tt)+) => (
        $crate::log!($crate::log::Loglevel::Debug, $($arg)+);
    )
}

#[macro_export]
macro_rules! trace {
    (target: $target:expr, $($arg:tt)+) => (
        $crate::log!(target: $target, $crate::log::Loglevel::Trace, $($arg)+);
    );
    ($($arg:tt)+) => (
        $crate::log!($crate::log::Loglevel::Trace, $($arg)+);
    )
}

#[cfg(test)]
mod tests {
    use super::{Loglevel, LoglevelFilter};

    #[test]
    fn level_order() {
        assert!(Loglevel::Debug < Loglevel::Trace);
        assert!(Loglevel::Info < Loglevel::Debug);
        assert!(Loglevel::Note < Loglevel::Info);
        assert!(Loglevel::Warn < Loglevel::Note);
        assert!(Loglevel::Error < Loglevel::Warn);
        assert!(Loglevel::Fatal < Loglevel::Error);
        assert!(LoglevelFilter::Off < LoglevelFilter::from(Loglevel::Fatal));
    }

}
