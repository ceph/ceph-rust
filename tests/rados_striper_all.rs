#![allow(unused_imports)]

extern crate ceph;
extern crate libc;

use std::env;
use std::ffi::CString;
use std::ptr;

use ceph::rados::*;
#[cfg(feature = "rados_striper")]
use ceph::rados_striper::*;

#[test]
#[cfg(feature = "rados_striper")]
fn test_rados_striper() {
    // This test only checks that all functions defined in rados_striper.rs can
    // be linked with libradosstriper.
    if env::var("TEST_RADOS_STRIPER_IGNORED").is_err() {
        return;
    }

    let obj_name = CString::new("testobject").unwrap();
    let obj_name_ptr = obj_name.as_ptr();
    let xattr_name = CString::new("testattr").unwrap();
    let xattr_name_ptr = xattr_name.as_ptr();
    let mut buf: Vec<u8> = vec![0; 1024];
    let buf_ptr = buf.as_ptr() as *const ::libc::c_char;

    unsafe {
        let ioctx: rados_ioctx_t = ptr::null_mut();

        let mut rados_striper: rados_striper_t = ptr::null_mut();
        rados_striper_create(ioctx, &mut rados_striper);

        rados_striper_destroy(rados_striper);

        rados_striper_set_object_layout_stripe_unit(rados_striper, 0 as ::libc::c_uint);

        rados_striper_set_object_layout_stripe_count(rados_striper, 0 as ::libc::c_uint);

        rados_striper_set_object_layout_object_size(rados_striper, 0 as ::libc::c_uint);

        rados_striper_write(rados_striper, obj_name_ptr, buf_ptr, 1024, 0);

        rados_striper_write_full(rados_striper, obj_name_ptr, buf_ptr, 1024);

        rados_striper_append(rados_striper, obj_name_ptr, buf_ptr, 1024);

        rados_striper_read(
            rados_striper,
            obj_name_ptr,
            buf.as_mut_ptr() as *mut ::libc::c_char,
            1024,
            0,
        );

        rados_striper_remove(rados_striper, obj_name_ptr);

        rados_striper_trunc(rados_striper, obj_name_ptr, 1024);

        rados_striper_getxattr(
            rados_striper,
            obj_name_ptr,
            xattr_name_ptr,
            buf.as_mut_ptr() as *mut ::libc::c_char,
            1024,
        );

        rados_striper_setxattr(rados_striper, obj_name_ptr, xattr_name_ptr, buf_ptr, 1024);

        rados_striper_rmxattr(rados_striper, obj_name_ptr, xattr_name_ptr);

        let mut xattr_iterator_handle: rados_xattrs_iter_t = ptr::null_mut();

        rados_striper_getxattrs(rados_striper, obj_name_ptr, &mut xattr_iterator_handle);

        let mut xattr_name_buf: Vec<u8> = vec![0; 1024];
        let mut xattr_value_buf: Vec<u8> = vec![0; 1024];
        let mut xattr_len: ::libc::size_t = 0;

        rados_striper_getxattrs_next(
            xattr_iterator_handle,
            xattr_name_buf.as_mut_ptr() as *mut *const ::libc::c_char,
            xattr_value_buf.as_mut_ptr() as *mut *const ::libc::c_char,
            &mut xattr_len,
        );

        rados_striper_getxattrs_end(xattr_iterator_handle);

        let mut psize: u64 = 0;
        let mut time: ::libc::time_t = 0;

        rados_striper_stat(rados_striper, obj_name_ptr, &mut psize, &mut time);

        let mut cb_arg: u8 = 0;
        let cb_complete: rados_callback_t = None;
        let cb_safe: rados_callback_t = None;
        let mut completion: rados_striper_multi_completion_t = ptr::null_mut();

        rados_striper_multi_aio_create_completion(
            &mut cb_arg as *mut _ as *mut ::std::os::raw::c_void,
            cb_complete,
            cb_safe,
            &mut completion,
        );
        rados_striper_multi_aio_wait_for_complete(completion);
        rados_striper_multi_aio_wait_for_safe(completion);
        rados_striper_multi_aio_is_complete(completion);
        rados_striper_multi_aio_is_safe(completion);
        rados_striper_multi_aio_wait_for_complete_and_cb(completion);
        rados_striper_multi_aio_wait_for_safe_and_cb(completion);
        rados_striper_multi_aio_is_complete_and_cb(completion);
        rados_striper_multi_aio_is_safe_and_cb(completion);
        rados_striper_multi_aio_get_return_value(completion);
        rados_striper_multi_aio_release(completion);

        rados_striper_aio_write(rados_striper, obj_name_ptr, completion, buf_ptr, 1024, 0);
        rados_striper_aio_append(rados_striper, obj_name_ptr, completion, buf_ptr, 1024);
        rados_striper_aio_write_full(rados_striper, obj_name_ptr, completion, buf_ptr, 1024);
        rados_striper_aio_read(
            rados_striper,
            obj_name_ptr,
            completion,
            buf.as_mut_ptr() as *mut ::libc::c_char,
            1024,
            0,
        );
        rados_striper_aio_remove(rados_striper, obj_name_ptr, completion);
        rados_striper_aio_flush(rados_striper);
        rados_striper_aio_stat(
            rados_striper,
            obj_name_ptr,
            completion,
            &mut psize,
            &mut time,
        );
    }
}
