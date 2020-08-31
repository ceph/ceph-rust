// The MIT License (MIT)
//
// Copyright (c) 2019 Luka Zakrajšek
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

#![allow(non_camel_case_types)]
#![allow(unused_imports)]

extern crate libc;

use self::libc::{size_t, ssize_t, time_t, timeval};

use super::rados::{rados_callback_t, rados_completion_t, rados_ioctx_t, rados_xattrs_iter_t};

pub type rados_striper_t = *mut ::std::os::raw::c_void;

pub type rados_striper_multi_completion_t = *mut ::std::os::raw::c_void;

#[cfg(unix)]
#[cfg(feature = "rados_striper")]
#[link(name = "radosstriper", kind = "dylib")]
extern "C" {
    pub fn rados_striper_create(
        ioctx: rados_ioctx_t,
        striper: *mut rados_striper_t,
    ) -> ::libc::c_int;

    pub fn rados_striper_destroy(striper: rados_striper_t) -> ();

    pub fn rados_striper_set_object_layout_stripe_unit(
        striper: rados_striper_t,
        stripe_unit: ::libc::c_uint,
    ) -> ::libc::c_int;

    pub fn rados_striper_set_object_layout_stripe_count(
        striper: rados_striper_t,
        stripe_count: ::libc::c_uint,
    ) -> ::libc::c_int;

    pub fn rados_striper_set_object_layout_object_size(
        striper: rados_striper_t,
        object_size: ::libc::c_uint,
    ) -> ::libc::c_int;

    pub fn rados_striper_write(
        striper: rados_striper_t,
        soid: *const ::libc::c_char,
        buf: *const ::libc::c_char,
        len: size_t,
        off: u64,
    ) -> ::libc::c_int;

    pub fn rados_striper_write_full(
        striper: rados_striper_t,
        soid: *const ::libc::c_char,
        buf: *const ::libc::c_char,
        len: size_t,
    ) -> ::libc::c_int;

    pub fn rados_striper_append(
        striper: rados_striper_t,
        soid: *const ::libc::c_char,
        buf: *const ::libc::c_char,
        len: size_t,
    ) -> ::libc::c_int;

    pub fn rados_striper_read(
        striper: rados_striper_t,
        soid: *const ::libc::c_char,
        buf: *mut ::libc::c_char,
        len: size_t,
        off: u64,
    ) -> ::libc::c_int;

    pub fn rados_striper_remove(
        striper: rados_striper_t,
        soid: *const ::libc::c_char,
    ) -> ::libc::c_int;

    pub fn rados_striper_trunc(
        striper: rados_striper_t,
        soid: *const ::libc::c_char,
        size: u64,
    ) -> ::libc::c_int;

    pub fn rados_striper_getxattr(
        striper: rados_striper_t,
        soid: *const ::libc::c_char,
        name: *const ::libc::c_char,
        buf: *mut ::libc::c_char,
        len: size_t,
    ) -> ::libc::c_int;

    pub fn rados_striper_setxattr(
        striper: rados_striper_t,
        soid: *const ::libc::c_char,
        name: *const ::libc::c_char,
        buf: *const ::libc::c_char,
        len: size_t,
    ) -> ::libc::c_int;

    pub fn rados_striper_rmxattr(
        striper: rados_striper_t,
        soid: *const ::libc::c_char,
        name: *const ::libc::c_char,
    ) -> ::libc::c_int;

    pub fn rados_striper_getxattrs(
        striper: rados_striper_t,
        soid: *const ::libc::c_char,
        iter: *mut rados_xattrs_iter_t,
    ) -> ::libc::c_int;

    pub fn rados_striper_getxattrs_next(
        iter: rados_xattrs_iter_t,
        name: *mut *const ::libc::c_char,
        val: *mut *const ::libc::c_char,
        len: *mut size_t,
    ) -> ::libc::c_int;

    pub fn rados_striper_getxattrs_end(iter: rados_xattrs_iter_t) -> ();

    pub fn rados_striper_stat(
        striper: rados_striper_t,
        soid: *const ::libc::c_char,
        psize: *mut u64,
        pmtime: *mut time_t,
    ) -> ::libc::c_int;

    pub fn rados_striper_multi_aio_create_completion(
        cb_arg: *mut ::std::os::raw::c_void,
        cb_complete: rados_callback_t,
        cb_safe: rados_callback_t,
        pc: *mut rados_striper_multi_completion_t,
    ) -> ::libc::c_int;
    pub fn rados_striper_multi_aio_wait_for_complete(
        c: rados_striper_multi_completion_t,
    ) -> ::libc::c_int;
    pub fn rados_striper_multi_aio_wait_for_safe(
        c: rados_striper_multi_completion_t,
    ) -> ::libc::c_int;
    pub fn rados_striper_multi_aio_is_complete(
        c: rados_striper_multi_completion_t,
    ) -> ::libc::c_int;
    pub fn rados_striper_multi_aio_is_safe(c: rados_striper_multi_completion_t) -> ::libc::c_int;
    pub fn rados_striper_multi_aio_wait_for_complete_and_cb(
        c: rados_striper_multi_completion_t,
    ) -> ::libc::c_int;
    pub fn rados_striper_multi_aio_wait_for_safe_and_cb(
        c: rados_striper_multi_completion_t,
    ) -> ::libc::c_int;
    pub fn rados_striper_multi_aio_is_complete_and_cb(
        c: rados_striper_multi_completion_t,
    ) -> ::libc::c_int;
    pub fn rados_striper_multi_aio_is_safe_and_cb(
        c: rados_striper_multi_completion_t,
    ) -> ::libc::c_int;
    pub fn rados_striper_multi_aio_get_return_value(
        c: rados_striper_multi_completion_t,
    ) -> ::libc::c_int;
    pub fn rados_striper_multi_aio_release(c: rados_striper_multi_completion_t) -> ();

    pub fn rados_striper_aio_write(
        striper: rados_striper_t,
        soid: *const ::libc::c_char,
        completion: rados_completion_t,
        buf: *const ::libc::c_char,
        len: size_t,
        off: u64,
    ) -> ::libc::c_int;
    pub fn rados_striper_aio_append(
        striper: rados_striper_t,
        soid: *const ::libc::c_char,
        completion: rados_completion_t,
        buf: *const ::libc::c_char,
        len: size_t,
    ) -> ::libc::c_int;
    pub fn rados_striper_aio_write_full(
        striper: rados_striper_t,
        soid: *const ::libc::c_char,
        completion: rados_completion_t,
        buf: *const ::libc::c_char,
        len: size_t,
    ) -> ::libc::c_int;
    pub fn rados_striper_aio_read(
        striper: rados_striper_t,
        soid: *const ::libc::c_char,
        completion: rados_completion_t,
        buf: *mut ::libc::c_char,
        len: size_t,
        off: u64,
    ) -> ::libc::c_int;
    pub fn rados_striper_aio_remove(
        striper: rados_striper_t,
        soid: *const ::libc::c_char,
        completion: rados_completion_t,
    ) -> ::libc::c_int;
    pub fn rados_striper_aio_flush(striper: rados_striper_t) -> ::libc::c_int;
    pub fn rados_striper_aio_stat(
        striper: rados_striper_t,
        o: *const ::libc::c_char,
        completion: rados_completion_t,
        psize: *mut u64,
        pmtime: *mut time_t,
    ) -> ::libc::c_int;
}
