use errors::*;
use libc::{c_int, c_char, strerror_r};

pub(crate) fn get_error(n: c_int) -> Result<String> {
    let mut buf = vec![0u8; 256];
    unsafe {
        strerror_r(n, buf.as_mut_ptr() as *mut c_char, buf.len());
        let message = String::from_utf8_lossy(&buf).into_owned();
        Ok(message)
    }
}