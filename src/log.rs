use std::{
    panic::Location,
    sync::atomic::{AtomicUsize, Ordering},
};

static LOG_LEVEL: AtomicUsize = AtomicUsize::new(LogLevel::Diagnostic as usize);

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Off,
    Fatal,
    Error,
    Warning,
    Diagnostic,
    Exec,
    Verbose,
    Info,
    Trace,
    Debug,
}

#[inline]
pub fn current_level() -> LogLevel {
    match LOG_LEVEL.load(Ordering::Relaxed) & (!0 >> 1) {
        0 => LogLevel::Off,
        1 => LogLevel::Fatal,
        2 => LogLevel::Error,
        3 => LogLevel::Warning,
        4 => LogLevel::Diagnostic,
        5 => LogLevel::Exec,
        6 => LogLevel::Verbose,
        7 => LogLevel::Info,
        8 => LogLevel::Trace,
        9 => LogLevel::Debug,
        val => panic!("Unknown logging level {}", val),
    }
}

#[inline]
pub fn set_logging_level(level: LogLevel) {
    let val = level as usize;
    let _ = LOG_LEVEL.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |prev| {
        Some((prev & !(!0 >> 1)) | val)
    });
}

#[inline]
pub fn use_term_formatting(use_term_fmt: bool) {
    let _ = LOG_LEVEL.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |prev| {
        Some((prev & !(!0 >> 1)) | ((use_term_fmt as usize) << (usize::BITS - 1)))
    });
}

#[inline]
pub fn set_logging_config(level: LogLevel, use_term_fmt: bool) {
    LOG_LEVEL.store(
        (level as usize) | (use_term_fmt as usize) << (usize::BITS - 1),
        Ordering::Relaxed,
    )
}

fn print_color_code(n: u8, f: &mut core::fmt::Formatter, use_term_fmt: bool) -> core::fmt::Result {
    if use_term_fmt {
        let code = if n < 8 { 30 + n } else { 90 + (n & 7) };
        f.write_fmt(format_args!("\x1B[{:02}m", code))
    } else {
        Ok(())
    }
}

fn print_reset(f: &mut core::fmt::Formatter, use_term_fmt: bool) -> core::fmt::Result {
    if use_term_fmt {
        f.write_str("\x1B[0m")
    } else {
        Ok(())
    }
}

impl core::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.display_prefix(f, (LOG_LEVEL.load(Ordering::Relaxed) & !(!0 >> 1)) != 0)
    }
}

impl LogLevel {
    pub fn display_prefix(
        &self,
        f: &mut core::fmt::Formatter,
        use_term_fmt: bool,
    ) -> core::fmt::Result {
        match self {
            LogLevel::Off => panic!("Logging at level `Off` is forbidden"),
            LogLevel::Fatal => {
                f.write_str("[")?;
                print_color_code(1, f, use_term_fmt)?;
                f.write_str("FATAL")?;
                print_reset(f, use_term_fmt)?;
                f.write_str("] ")
            }
            LogLevel::Error => {
                f.write_str("[")?;
                print_color_code(9, f, use_term_fmt)?;
                f.write_str("ERROR")?;
                print_reset(f, use_term_fmt)?;
                f.write_str("] ")
            }
            LogLevel::Warning => {
                f.write_str("[")?;
                print_color_code(11, f, use_term_fmt)?;
                f.write_str("WARN")?;
                print_reset(f, use_term_fmt)?;
                f.write_str("] ")
            }
            LogLevel::Diagnostic => f.write_str("[DIAGNOSTIC] "),
            LogLevel::Exec => f.write_str("[EXEC] "),
            LogLevel::Verbose => Ok(()),
            LogLevel::Info => {
                f.write_str("[")?;
                print_color_code(7, f, use_term_fmt)?;
                f.write_str("INFO")?;
                print_reset(f, use_term_fmt)?;
                f.write_str("] ")
            }
            LogLevel::Trace => {
                f.write_str("[")?;
                print_color_code(8, f, use_term_fmt)?;
                f.write_str("TRACE")?;
                print_reset(f, use_term_fmt)?;
                f.write_str("] ")
            }
            LogLevel::Debug => {
                f.write_str("[")?;
                print_color_code(7, f, use_term_fmt)?;
                f.write_str("DEBUG")?;
                print_reset(f, use_term_fmt)?;
                f.write_str("] ")
            }
        }
    }
}

#[doc(hidden)]
#[inline]
pub fn __log_print(level: LogLevel, f: core::fmt::Arguments) {
    use std::io::Write;
    if level <= current_level() {
        writeln!(std::io::stderr(), "{}{}", level, f).expect("Error writing to STDERR stream")
    }
}

#[doc(hidden)]
#[inline]
pub fn __log_debug_print(level: LogLevel, f: core::fmt::Arguments, loc: &Location) {
    use std::io::Write;
    if level <= current_level() {
        writeln!(std::io::stderr(), "{}[{}]: {}", level, loc, f)
            .expect("Error writing to STDERR stream")
    }
}

macro_rules! log{
    ($level:expr, $($fmt:tt)*) => {
        $crate::log::__log_print($level, ::core::format_args!($($fmt)*))
    };
}

pub(crate) use log;

macro_rules! log_debug{
    ($level:expr, $($fmt:tt)*) => {
        $crate::log::__log_debug_print($level, ::core::format_args!($($fmt)*), ::core::panic::Location::caller())
    };
}

pub(crate) use log_debug;

macro_rules! trace {
    ($($fn_name:ident)::+) => {
        let __guard = {
            struct __PrintTrace;
            impl core::ops::Drop for __PrintTrace {
                fn drop(&mut self) {
                    if std::thread::panicking() {
                        $crate::log::__log_debug_print(
                            $crate::log::LogLevel::Trace,
                            ::core::format_args!("{{{}}}: unwind", ::core::concat!("" $(,::core::stringify!($fn_name),)"::"+)),
                            ::core::panic::Location::caller(),
                        )
                    } else {
                        $crate::log::__log_debug_print(
                            $crate::log::LogLevel::Trace,
                            ::core::format_args!("{{{}}}: return", ::core::concat!("" $(,::core::stringify!($fn_name),)"::"+)),
                            ::core::panic::Location::caller(),
                        )
                    }
                }
            }
            $crate::log::__log_debug_print(
                $crate::log::LogLevel::Trace,
                ::core::format_args!("{{{}}}: entry", ::core::concat!("" $(,::core::stringify!($fn_name),)"::"+)),
                ::core::panic::Location::caller(),
            );
            __PrintTrace
        };
    };
}

pub(crate) use trace;

macro_rules! dbg {
    (&$expr:expr) => {{
        let __expr = &$expr;
        $crate::log::__log_debug_print(
            $crate::log::LogLevel::Debug,
            ::core::format_args!("{} = {:#?}", ::core::stringify!($expr), __expr),
            ::core::panic::Location::caller(),
        );
        __expr
    }};
    ($expr:expr) => {{
        let __expr = $expr;
        $crate::log::__log_debug_print(
            $crate::log::LogLevel::Debug,
            ::core::format_args!("{} = {:#?}", ::core::stringify!($expr), __expr),
            ::core::panic::Location::caller(),
        );
        __expr
    }};
}

pub(crate) use dbg;
