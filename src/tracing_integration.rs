use crate::error::RadosResult;
use crate::rados::*;

use libc::*;
use std::ffi::CStr;
use tracing::Level;

/// For tracing integration to function properly, logging (tracing handler) must be enabled before
/// connecting to ceph: if tracing is not enabled, we don't set a callback, which avoids the
/// corresponding overhead.
pub(crate) fn enable_tracing_integration(rados: rados_t) -> RadosResult<()> {
    let (level_str, opt_log_callback): (&[u8], rados_log_callback_t) = {
        if event_enabled!(target: "librados", Level::ERROR) {
            if event_enabled!(target: "librados", Level::WARN) {
                if event_enabled!(target: "librados", Level::INFO) {
                    if event_enabled!(target: "librados", Level::DEBUG) {
                        (b"debug\0", Some(log_callback))
                    } else {
                        (b"info\0", Some(log_callback))
                    }
                } else {
                    (b"warn\0", Some(log_callback))
                }
            } else {
                (b"error\0", Some(log_callback))
            }
        } else {
            (b"\0", None)
        }
    };

    let level_cstr =
        CStr::from_bytes_with_nul(level_str).expect("Constructed with trailing nul byte above");
    unsafe {
        let ret_code = rados_monitor_log(
            rados,
            level_cstr.as_ptr(),
            opt_log_callback,
            0 as *mut c_void,
        );
        if ret_code < 0 {
            return Err(ret_code.into());
        }
    }

    Ok(())
}

extern "C" fn log_callback(
    _arg: *mut ::std::os::raw::c_void,
    line: *const ::libc::c_char,
    who: *const ::libc::c_char,
    _sec: u64,
    _nsec: u64,
    _seq: u64,
    level: *const ::libc::c_char,
    msg: *const ::libc::c_char,
) {
    macro_rules! level {
        ($($l: expr => $n: ident,)*) => {
            match unsafe { *((level as *const u8).add(1)) } {
                $(
                    $l => {
                        if event_enabled!(target: "librados", Level::$n) {
                            Level::$n
                        } else {
                            return;
                        }
                    }
                )*
                _ => {
                    let s = unsafe { CStr::from_ptr(level) };
                    let level = s.to_string_lossy();
                    event!(target: "librados", Level::WARN, "Unknown log level: {level} - please report issue at github.com/ceph/ceph-rust");
                    Level::DEBUG
                },
            }
        };
    }
    let level = level!(
        b'E' => ERROR,
        b'W' => WARN,
        b'I' => INFO,
        b'D' => DEBUG,
    );

    // We need to log, build the things
    let who = unsafe { CStr::from_ptr(who) };
    let who = who.to_string_lossy();
    let line = unsafe { CStr::from_ptr(line) };
    let line: Option<usize> = line.to_str().ok().and_then(|l| l.parse().ok());
    let msg = unsafe { CStr::from_ptr(msg) };
    let msg = msg.to_string_lossy();

    // Macro is necessary because tracing::event! requires level to be const
    // https://github.com/tokio-rs/tracing/issues/2730
    macro_rules! build_and_capture_event {
            ($($lvl: ident),*) => {
                match level {
                    $(
                        Level::$lvl => {
                            if let Some(line) = line {
                                event!(target: "librados", Level::$lvl, %who, %line, "{msg}");
                            } else {
                                event!(target: "librados", Level::$lvl, %who, "{msg}");
                            }
                        },
                    )*
                    _ => unreachable!(),
                }

            };
        }
    build_and_capture_event!(DEBUG, INFO, WARN, ERROR);
}
