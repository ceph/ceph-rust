// Copyright 2017 LambdaStack All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Portions from Chris Holcombe
// The MIT License (MIT)
//
// Copyright (c) 2015 Chris Holcombe
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

use self::libc::{int64_t, size_t, ssize_t, time_t, timeval, uint32_t, uint64_t, uint8_t};

bitflags! {
    pub flags AllocFlags: u32 {
        const LIBRADOS_OP_FLAG_EXCL = 1,
        const LIBRADOS_OP_FLAG_FAILOK = 2,
        const LIBRADOS_OP_FLAG_FADVISE_RANDOM = 4,
        const LIBRADOS_OP_FLAG_FADVISE_SEQUENTIAL = 8,
        const LIBRADOS_OP_FLAG_FADVISE_WILLNEED = 16,
        const LIBRADOS_OP_FLAG_FADVISE_DONTNEED = 32,
        const LIBRADOS_OP_FLAG_FADVISE_NOCACHE = 64,
    }
}
impl AllocFlags {
    pub fn clear(&mut self) {
        self.bits = 0; // The `bits` field can be accessed from within the
                       // same module where the `bitflags!` macro was invoked.
    }
}

bitflags! {
    pub flags XattrFlags: u32 {
        const LIBRADOS_CMPXATTR_OP_EQ = 1,
        const LIBRADOS_CMPXATTR_OP_NE = 2,
        const LIBRADOS_CMPXATTR_OP_GT = 3,
        const LIBRADOS_CMPXATTR_OP_GTE = 4,
        const LIBRADOS_CMPXATTR_OP_LT = 5,
        const LIBRADOS_CMPXATTR_OP_LTE = 6,
    }
}
impl XattrFlags {
    pub fn clear(&mut self) {
        self.bits = 0; // The `bits` field can be accessed from within the
                       // same module where the `bitflags!` macro was invoked.
    }
}

// Flags for rados_read_op_operate(), rados_write_op_operate(),
// rados_aio_read_op_operate(), and rados_aio_write_op_operate()
bitflags! {
    pub flags OperationFlags: u32 {
        const LIBRADOS_OPERATION_NOFLAG= 0,
        const LIBRADOS_OPERATION_BALANCE_READS= 1,
        const LIBRADOS_OPERATION_LOCALIZE_READS= 2,
        const LIBRADOS_OPERATION_ORDER_READS_WRITES= 4,
        const LIBRADOS_OPERATION_IGNORE_CACHE= 8,
        const LIBRADOS_OPERATION_SKIPRWLOCKS= 16,
        const LIBRADOS_OPERATION_IGNORE_OVERLAY= 32,
    }
}

impl OperationFlags {
    pub fn clear(&mut self) {
        self.bits = 0; // The `bits` field can be accessed from within the
                       // same module where the `bitflags!` macro was invoked.
    }
}

pub type rados_t = *mut ::std::os::raw::c_void;
pub type rados_config_t = *mut ::std::os::raw::c_void;
pub type rados_ioctx_t = *mut ::std::os::raw::c_void;
pub type rados_list_ctx_t = *mut ::std::os::raw::c_void;
pub type rados_snap_t = uint64_t;
pub type rados_xattrs_iter_t = *mut ::std::os::raw::c_void;
pub type rados_omap_iter_t = *mut ::std::os::raw::c_void;

#[repr(C)]
#[derive(Copy, Debug)]
pub struct Struct_rados_pool_stat_t {
    pub num_bytes: uint64_t,
    pub num_kb: uint64_t,
    pub num_objects: uint64_t,
    pub num_object_clones: uint64_t,
    pub num_object_copies: uint64_t,
    pub num_objects_missing_on_primary: uint64_t,
    pub num_objects_unfound: uint64_t,
    pub num_objects_degraded: uint64_t,
    pub num_rd: uint64_t,
    pub num_rd_kb: uint64_t,
    pub num_wr: uint64_t,
    pub num_wr_kb: uint64_t,
}

impl ::std::clone::Clone for Struct_rados_pool_stat_t {
    fn clone(&self) -> Self {
        *self
    }
}

impl ::std::default::Default for Struct_rados_pool_stat_t {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Copy, Debug)]
pub struct Struct_rados_cluster_stat_t {
    pub kb: uint64_t,
    pub kb_used: uint64_t,
    pub kb_avail: uint64_t,
    pub num_objects: uint64_t,
}

impl ::std::clone::Clone for Struct_rados_cluster_stat_t {
    fn clone(&self) -> Self {
        *self
    }
}

impl ::std::default::Default for Struct_rados_cluster_stat_t {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}

pub type rados_write_op_t = *mut ::std::os::raw::c_void;

pub type rados_read_op_t = *mut ::std::os::raw::c_void;

pub type rados_completion_t = *mut ::std::os::raw::c_void;

pub type rados_callback_t =
    ::std::option::Option<extern "C" fn(cb: rados_completion_t, arg: *mut ::std::os::raw::c_void) -> ()>;

pub type rados_watchcb_t =
    ::std::option::Option<extern "C" fn(opcode: uint8_t, ver: uint64_t, arg: *mut ::std::os::raw::c_void) -> ()>;

pub type rados_watchcb2_t = ::std::option::Option<
    extern "C" fn(
        arg: *mut ::std::os::raw::c_void,
        notify_id: uint64_t,
        handle: uint64_t,
        notifier_id: uint64_t,
        data: *mut ::std::os::raw::c_void,
        data_len: size_t,
    ) -> (),
>;

pub type rados_watcherrcb_t =
    ::std::option::Option<extern "C" fn(pre: *mut ::std::os::raw::c_void, cookie: uint64_t, err: ::libc::c_int) -> ()>;

pub type rados_log_callback_t = ::std::option::Option<
    extern "C" fn(
        arg: *mut ::std::os::raw::c_void,
        line: *const ::libc::c_char,
        who: *const ::libc::c_char,
        sec: uint64_t,
        nsec: uint64_t,
        seq: uint64_t,
        level: *const ::libc::c_char,
        msg: *const ::libc::c_char,
    ) -> (),
>;

#[cfg(target_os = "linux")]
#[link(name = "rados", kind = "dylib")]
extern "C" {
    pub fn rados_version(major: *mut ::libc::c_int, minor: *mut ::libc::c_int, extra: *mut ::libc::c_int) -> ();

    pub fn rados_create(cluster: *mut rados_t, id: *const ::libc::c_char) -> ::libc::c_int;

    pub fn rados_create2(
        pcluster: *mut rados_t,
        clustername: *const ::libc::c_char,
        name: *const ::libc::c_char,
        flags: uint64_t,
    ) -> ::libc::c_int;

    pub fn rados_create_with_context(cluster: *mut rados_t, cct: rados_config_t) -> ::libc::c_int;

    pub fn rados_ping_monitor(
        cluster: rados_t,
        mon_id: *const ::libc::c_char,
        outstr: *mut *mut ::libc::c_char,
        outstrlen: *mut size_t,
    ) -> ::libc::c_int;

    pub fn rados_connect(cluster: rados_t) -> ::libc::c_int;

    pub fn rados_shutdown(cluster: rados_t) -> ();

    pub fn rados_conf_read_file(cluster: rados_t, path: *const ::libc::c_char) -> ::libc::c_int;

    pub fn rados_conf_parse_argv(
        cluster: rados_t,
        argc: ::libc::c_int,
        argv: *mut *const ::libc::c_char,
    ) -> ::libc::c_int;

    pub fn rados_conf_parse_argv_remainder(
        cluster: rados_t,
        argc: ::libc::c_int,
        argv: *mut *const ::libc::c_char,
        remargv: *mut *const ::libc::c_char,
    ) -> ::libc::c_int;

    pub fn rados_conf_parse_env(cluster: rados_t, var: *const ::libc::c_char) -> ::libc::c_int;

    pub fn rados_conf_set(
        cluster: rados_t,
        option: *const ::libc::c_char,
        value: *const ::libc::c_char,
    ) -> ::libc::c_int;

    pub fn rados_conf_get(
        cluster: rados_t,
        option: *const ::libc::c_char,
        buf: *mut ::libc::c_char,
        len: size_t,
    ) -> ::libc::c_int;

    pub fn rados_cluster_stat(cluster: rados_t, result: *mut Struct_rados_cluster_stat_t) -> ::libc::c_int;

    /// The Ceph documentation states that the return value of 0 means success
    /// but actually a value < 0 is an error and a value > 0 is the length
    /// which should be 36.
    ///
    pub fn rados_cluster_fsid(cluster: rados_t, buf: *mut ::libc::c_char, len: size_t) -> ::libc::c_int;

    pub fn rados_wait_for_latest_osdmap(cluster: rados_t) -> ::libc::c_int;

    pub fn rados_pool_list(cluster: rados_t, buf: *mut ::libc::c_char, len: size_t) -> ::libc::c_int;

    pub fn rados_cct(cluster: rados_t) -> rados_config_t;

    pub fn rados_get_instance_id(cluster: rados_t) -> uint64_t;

    pub fn rados_ioctx_create(
        cluster: rados_t,
        pool_name: *const ::libc::c_char,
        ioctx: *mut rados_ioctx_t,
    ) -> ::libc::c_int;

    pub fn rados_ioctx_create2(cluster: rados_t, pool_id: int64_t, ioctx: *mut rados_ioctx_t) -> ::libc::c_int;

    pub fn rados_ioctx_destroy(io: rados_ioctx_t) -> ();

    pub fn rados_ioctx_cct(io: rados_ioctx_t) -> rados_config_t;

    pub fn rados_ioctx_get_cluster(io: rados_ioctx_t) -> rados_t;

    pub fn rados_ioctx_pool_stat(io: rados_ioctx_t, stats: *mut Struct_rados_pool_stat_t) -> ::libc::c_int;

    pub fn rados_pool_lookup(cluster: rados_t, pool_name: *const ::libc::c_char) -> int64_t;

    pub fn rados_pool_reverse_lookup(
        cluster: rados_t,
        id: int64_t,
        buf: *mut ::libc::c_char,
        maxlen: size_t,
    ) -> ::libc::c_int;

    /// Returns -17 if pool already exists.
    pub fn rados_pool_create(cluster: rados_t, pool_name: *const ::libc::c_char) -> ::libc::c_int;

    pub fn rados_pool_create_with_auid(
        cluster: rados_t,
        pool_name: *const ::libc::c_char,
        auid: uint64_t,
    ) -> ::libc::c_int;

    pub fn rados_pool_create_with_crush_rule(
        cluster: rados_t,
        pool_name: *const ::libc::c_char,
        crush_rule_num: uint8_t,
    ) -> ::libc::c_int;

    pub fn rados_pool_create_with_all(
        cluster: rados_t,
        pool_name: *const ::libc::c_char,
        auid: uint64_t,
        crush_rule_num: uint8_t,
    ) -> ::libc::c_int;

    pub fn rados_pool_get_base_tier(cluster: rados_t, pool: int64_t, base_tier: *mut int64_t) -> ::libc::c_int;

    pub fn rados_pool_delete(cluster: rados_t, pool_name: *const ::libc::c_char) -> ::libc::c_int;

    pub fn rados_ioctx_pool_set_auid(io: rados_ioctx_t, auid: uint64_t) -> ::libc::c_int;

    pub fn rados_ioctx_pool_get_auid(io: rados_ioctx_t, auid: *mut uint64_t) -> ::libc::c_int;

    pub fn rados_ioctx_pool_requires_alignment(io: rados_ioctx_t) -> ::libc::c_int;

    pub fn rados_ioctx_pool_required_alignment(io: rados_ioctx_t) -> uint64_t;

    pub fn rados_ioctx_get_id(io: rados_ioctx_t) -> int64_t;

    pub fn rados_ioctx_get_pool_name(
        io: rados_ioctx_t,
        buf: *mut ::libc::c_char,
        maxlen: ::libc::c_uint,
    ) -> ::libc::c_int;

    pub fn rados_ioctx_locator_set_key(io: rados_ioctx_t, key: *const ::libc::c_char) -> ();

    pub fn rados_ioctx_set_namespace(io: rados_ioctx_t, nspace: *const ::libc::c_char) -> ();

    pub fn rados_nobjects_list_open(io: rados_ioctx_t, ctx: *mut rados_list_ctx_t) -> ::libc::c_int;

    pub fn rados_nobjects_list_get_pg_hash_position(ctx: rados_list_ctx_t) -> uint32_t;

    pub fn rados_nobjects_list_seek(ctx: rados_list_ctx_t, pos: uint32_t) -> uint32_t;

    pub fn rados_nobjects_list_next(
        ctx: rados_list_ctx_t,
        entry: *mut *mut *const ::libc::c_char,
        key: *mut *mut *const ::libc::c_char,
        nspace: *mut *mut *const ::libc::c_char,
    ) -> ::libc::c_int;

    pub fn rados_nobjects_list_close(ctx: rados_list_ctx_t) -> ();

    pub fn rados_objects_list_open(io: rados_ioctx_t, ctx: *mut rados_list_ctx_t) -> ::libc::c_int;

    pub fn rados_objects_list_get_pg_hash_position(ctx: rados_list_ctx_t) -> uint32_t;

    pub fn rados_objects_list_seek(ctx: rados_list_ctx_t, pos: uint32_t) -> uint32_t;

    pub fn rados_objects_list_next(
        ctx: rados_list_ctx_t,
        entry: *mut *const ::libc::c_char,
        key: *mut *const ::libc::c_char,
    ) -> ::libc::c_int;

    pub fn rados_objects_list_close(ctx: rados_list_ctx_t) -> ();

    pub fn rados_ioctx_snap_create(io: rados_ioctx_t, snapname: *const ::libc::c_char) -> ::libc::c_int;

    pub fn rados_ioctx_snap_remove(io: rados_ioctx_t, snapname: *const ::libc::c_char) -> ::libc::c_int;

    pub fn rados_ioctx_snap_rollback(
        io: rados_ioctx_t,
        oid: *const ::libc::c_char,
        snapname: *const ::libc::c_char,
    ) -> ::libc::c_int;
    // Deprecated: Use rados_ioctx_snap_rollback() instead
    pub fn rados_rollback(
        io: rados_ioctx_t,
        oid: *const ::libc::c_char,
        snapname: *const ::libc::c_char,
    ) -> ::libc::c_int;

    pub fn rados_ioctx_snap_set_read(io: rados_ioctx_t, snap: rados_snap_t) -> ();

    pub fn rados_ioctx_selfmanaged_snap_create(io: rados_ioctx_t, snapid: *mut rados_snap_t) -> ::libc::c_int;

    pub fn rados_ioctx_selfmanaged_snap_remove(io: rados_ioctx_t, snapid: rados_snap_t) -> ::libc::c_int;

    pub fn rados_ioctx_selfmanaged_snap_rollback(
        io: rados_ioctx_t,
        oid: *const ::libc::c_char,
        snapid: rados_snap_t,
    ) -> ::libc::c_int;

    pub fn rados_ioctx_selfmanaged_snap_set_write_ctx(
        io: rados_ioctx_t,
        seq: rados_snap_t,
        snaps: *mut rados_snap_t,
        num_snaps: ::libc::c_int,
    ) -> ::libc::c_int;

    pub fn rados_ioctx_snap_list(io: rados_ioctx_t, snaps: *mut rados_snap_t, maxlen: ::libc::c_int) -> ::libc::c_int;

    pub fn rados_ioctx_snap_lookup(
        io: rados_ioctx_t,
        name: *const ::libc::c_char,
        id: *mut rados_snap_t,
    ) -> ::libc::c_int;

    pub fn rados_ioctx_snap_get_name(
        io: rados_ioctx_t,
        id: rados_snap_t,
        name: *mut ::libc::c_char,
        maxlen: ::libc::c_int,
    ) -> ::libc::c_int;

    pub fn rados_ioctx_snap_get_stamp(io: rados_ioctx_t, id: rados_snap_t, t: *mut time_t) -> ::libc::c_int;

    pub fn rados_get_last_version(io: rados_ioctx_t) -> uint64_t;

    pub fn rados_write(
        io: rados_ioctx_t,
        oid: *const ::libc::c_char,
        buf: *const ::libc::c_char,
        len: size_t,
        off: uint64_t,
    ) -> ::libc::c_int;

    pub fn rados_write_full(
        io: rados_ioctx_t,
        oid: *const ::libc::c_char,
        buf: *const ::libc::c_char,
        len: size_t,
    ) -> ::libc::c_int;

    pub fn rados_clone_range(
        io: rados_ioctx_t,
        dst: *const ::libc::c_char,
        dst_off: uint64_t,
        src: *const ::libc::c_char,
        src_off: uint64_t,
        len: size_t,
    ) -> ::libc::c_int;

    pub fn rados_append(
        io: rados_ioctx_t,
        oid: *const ::libc::c_char,
        buf: *const ::libc::c_char,
        len: size_t,
    ) -> ::libc::c_int;

    pub fn rados_read(
        io: rados_ioctx_t,
        oid: *const ::libc::c_char,
        buf: *mut ::libc::c_char,
        len: size_t,
        off: uint64_t,
    ) -> ::libc::c_int;

    pub fn rados_remove(io: rados_ioctx_t, oid: *const ::libc::c_char) -> ::libc::c_int;

    pub fn rados_trunc(io: rados_ioctx_t, oid: *const ::libc::c_char, size: uint64_t) -> ::libc::c_int;

    pub fn rados_getxattr(
        io: rados_ioctx_t,
        o: *const ::libc::c_char,
        name: *const ::libc::c_char,
        buf: *mut ::libc::c_char,
        len: size_t,
    ) -> ::libc::c_int;

    pub fn rados_setxattr(
        io: rados_ioctx_t,
        o: *const ::libc::c_char,
        name: *const ::libc::c_char,
        buf: *const ::libc::c_char,
        len: size_t,
    ) -> ::libc::c_int;

    pub fn rados_rmxattr(io: rados_ioctx_t, o: *const ::libc::c_char, name: *const ::libc::c_char) -> ::libc::c_int;

    pub fn rados_getxattrs(
        io: rados_ioctx_t,
        oid: *const ::libc::c_char,
        iter: *mut rados_xattrs_iter_t,
    ) -> ::libc::c_int;

    pub fn rados_getxattrs_next(
        iter: rados_xattrs_iter_t,
        name: *mut *const ::libc::c_char,
        val: *mut *const ::libc::c_char,
        len: *mut size_t,
    ) -> ::libc::c_int;

    pub fn rados_getxattrs_end(iter: rados_xattrs_iter_t) -> ();

    pub fn rados_omap_get_next(
        iter: rados_omap_iter_t,
        key: *mut *mut ::libc::c_char,
        val: *mut *mut ::libc::c_char,
        len: *mut size_t,
    ) -> ::libc::c_int;

    pub fn rados_omap_get_end(iter: rados_omap_iter_t) -> ();

    pub fn rados_stat(
        io: rados_ioctx_t,
        o: *const ::libc::c_char,
        psize: *mut uint64_t,
        pmtime: *mut time_t,
    ) -> ::libc::c_int;

    pub fn rados_tmap_update(
        io: rados_ioctx_t,
        o: *const ::libc::c_char,
        cmdbuf: *const ::libc::c_char,
        cmdbuflen: size_t,
    ) -> ::libc::c_int;

    pub fn rados_tmap_put(
        io: rados_ioctx_t,
        o: *const ::libc::c_char,
        buf: *const ::libc::c_char,
        buflen: size_t,
    ) -> ::libc::c_int;

    pub fn rados_tmap_get(
        io: rados_ioctx_t,
        o: *const ::libc::c_char,
        buf: *mut ::libc::c_char,
        buflen: size_t,
    ) -> ::libc::c_int;

    pub fn rados_exec(
        io: rados_ioctx_t,
        oid: *const ::libc::c_char,
        cls: *const ::libc::c_char,
        method: *const ::libc::c_char,
        in_buf: *const ::libc::c_char,
        in_len: size_t,
        buf: *mut ::libc::c_char,
        out_len: size_t,
    ) -> ::libc::c_int;

    pub fn rados_aio_create_completion(
        cb_arg: *mut ::std::os::raw::c_void,
        cb_complete: rados_callback_t,
        cb_safe: rados_callback_t,
        pc: *mut rados_completion_t,
    ) -> ::libc::c_int;
    pub fn rados_aio_wait_for_complete(c: rados_completion_t) -> ::libc::c_int;
    pub fn rados_aio_wait_for_safe(c: rados_completion_t) -> ::libc::c_int;
    pub fn rados_aio_is_complete(c: rados_completion_t) -> ::libc::c_int;
    pub fn rados_aio_is_safe(c: rados_completion_t) -> ::libc::c_int;
    pub fn rados_aio_wait_for_complete_and_cb(c: rados_completion_t) -> ::libc::c_int;
    pub fn rados_aio_wait_for_safe_and_cb(c: rados_completion_t) -> ::libc::c_int;
    pub fn rados_aio_is_complete_and_cb(c: rados_completion_t) -> ::libc::c_int;
    pub fn rados_aio_is_safe_and_cb(c: rados_completion_t) -> ::libc::c_int;
    pub fn rados_aio_get_return_value(c: rados_completion_t) -> ::libc::c_int;
    pub fn rados_aio_release(c: rados_completion_t) -> ();
    pub fn rados_aio_write(
        io: rados_ioctx_t,
        oid: *const ::libc::c_char,
        completion: rados_completion_t,
        buf: *const ::libc::c_char,
        len: size_t,
        off: uint64_t,
    ) -> ::libc::c_int;
    pub fn rados_aio_append(
        io: rados_ioctx_t,
        oid: *const ::libc::c_char,
        completion: rados_completion_t,
        buf: *const ::libc::c_char,
        len: size_t,
    ) -> ::libc::c_int;
    pub fn rados_aio_write_full(
        io: rados_ioctx_t,
        oid: *const ::libc::c_char,
        completion: rados_completion_t,
        buf: *const ::libc::c_char,
        len: size_t,
    ) -> ::libc::c_int;
    pub fn rados_aio_remove(
        io: rados_ioctx_t,
        oid: *const ::libc::c_char,
        completion: rados_completion_t,
    ) -> ::libc::c_int;
    pub fn rados_aio_read(
        io: rados_ioctx_t,
        oid: *const ::libc::c_char,
        completion: rados_completion_t,
        buf: *mut ::libc::c_char,
        len: size_t,
        off: uint64_t,
    ) -> ::libc::c_int;
    pub fn rados_aio_flush(io: rados_ioctx_t) -> ::libc::c_int;
    pub fn rados_aio_flush_async(io: rados_ioctx_t, completion: rados_completion_t) -> ::libc::c_int;
    pub fn rados_aio_stat(
        io: rados_ioctx_t,
        o: *const ::libc::c_char,
        completion: rados_completion_t,
        psize: *mut uint64_t,
        pmtime: *mut time_t,
    ) -> ::libc::c_int;
    pub fn rados_aio_cancel(io: rados_ioctx_t, completion: rados_completion_t) -> ::libc::c_int;
    pub fn rados_watch(
        io: rados_ioctx_t,
        o: *const ::libc::c_char,
        ver: uint64_t,
        cookie: *mut uint64_t,
        watchcb: rados_watchcb_t,
        arg: *mut ::std::os::raw::c_void,
    ) -> ::libc::c_int;
    pub fn rados_watch2(
        io: rados_ioctx_t,
        o: *const ::libc::c_char,
        cookie: *mut uint64_t,
        watchcb: rados_watchcb2_t,
        watcherrcb: rados_watcherrcb_t,
        arg: *mut ::std::os::raw::c_void,
    ) -> ::libc::c_int;
    pub fn rados_watch_check(io: rados_ioctx_t, cookie: uint64_t) -> ::libc::c_int;
    pub fn rados_unwatch(io: rados_ioctx_t, o: *const ::libc::c_char, cookie: uint64_t) -> ::libc::c_int;
    pub fn rados_unwatch2(io: rados_ioctx_t, cookie: uint64_t) -> ::libc::c_int;
    pub fn rados_notify(
        io: rados_ioctx_t,
        o: *const ::libc::c_char,
        ver: uint64_t,
        buf: *const ::libc::c_char,
        buf_len: ::libc::c_int,
    ) -> ::libc::c_int;
    pub fn rados_notify2(
        io: rados_ioctx_t,
        o: *const ::libc::c_char,
        buf: *const ::libc::c_char,
        buf_len: ::libc::c_int,
        timeout_ms: uint64_t,
        reply_buffer: *mut *mut ::libc::c_char,
        reply_buffer_len: *mut size_t,
    ) -> ::libc::c_int;
    pub fn rados_notify_ack(
        io: rados_ioctx_t,
        o: *const ::libc::c_char,
        notify_id: uint64_t,
        cookie: uint64_t,
        buf: *const ::libc::c_char,
        buf_len: ::libc::c_int,
    ) -> ::libc::c_int;
    pub fn rados_watch_flush(cluster: rados_t) -> ::libc::c_int;
    pub fn rados_set_alloc_hint(
        io: rados_ioctx_t,
        o: *const ::libc::c_char,
        expected_object_size: uint64_t,
        expected_write_size: uint64_t,
    ) -> ::libc::c_int;
    pub fn rados_create_write_op() -> rados_write_op_t;
    pub fn rados_release_write_op(write_op: rados_write_op_t) -> ();
    pub fn rados_write_op_set_flags(write_op: rados_write_op_t, flags: ::libc::c_int) -> ();
    pub fn rados_write_op_assert_exists(write_op: rados_write_op_t) -> ();
    pub fn rados_write_op_assert_version(write_op: rados_write_op_t, ver: uint64_t) -> ();
    pub fn rados_write_op_cmpxattr(
        write_op: rados_write_op_t,
        name: *const ::libc::c_char,
        comparison_operator: uint8_t,
        value: *const ::libc::c_char,
        value_len: size_t,
    ) -> ();
    pub fn rados_write_op_omap_cmp(
        write_op: rados_write_op_t,
        key: *const ::libc::c_char,
        comparison_operator: uint8_t,
        val: *const ::libc::c_char,
        val_len: size_t,
        prval: *mut ::libc::c_int,
    ) -> ();
    pub fn rados_write_op_setxattr(
        write_op: rados_write_op_t,
        name: *const ::libc::c_char,
        value: *const ::libc::c_char,
        value_len: size_t,
    ) -> ();
    pub fn rados_write_op_rmxattr(write_op: rados_write_op_t, name: *const ::libc::c_char) -> ();
    pub fn rados_write_op_create(
        write_op: rados_write_op_t,
        exclusive: ::libc::c_int,
        category: *const ::libc::c_char,
    ) -> ();
    pub fn rados_write_op_write(
        write_op: rados_write_op_t,
        buffer: *const ::libc::c_char,
        len: size_t,
        offset: uint64_t,
    ) -> ();
    pub fn rados_write_op_write_full(write_op: rados_write_op_t, buffer: *const ::libc::c_char, len: size_t) -> ();
    pub fn rados_write_op_append(write_op: rados_write_op_t, buffer: *const ::libc::c_char, len: size_t) -> ();
    pub fn rados_write_op_remove(write_op: rados_write_op_t) -> ();
    pub fn rados_write_op_truncate(write_op: rados_write_op_t, offset: uint64_t) -> ();
    pub fn rados_write_op_zero(write_op: rados_write_op_t, offset: uint64_t, len: uint64_t) -> ();
    pub fn rados_write_op_exec(
        write_op: rados_write_op_t,
        cls: *const ::libc::c_char,
        method: *const ::libc::c_char,
        in_buf: *const ::libc::c_char,
        in_len: size_t,
        prval: *mut ::libc::c_int,
    ) -> ();
    pub fn rados_write_op_omap_set(
        write_op: rados_write_op_t,
        keys: *const *const ::libc::c_char,
        vals: *const *const ::libc::c_char,
        lens: *const size_t,
        num: size_t,
    ) -> ();
    pub fn rados_write_op_omap_rm_keys(
        write_op: rados_write_op_t,
        keys: *const *const ::libc::c_char,
        keys_len: size_t,
    ) -> ();
    pub fn rados_write_op_omap_clear(write_op: rados_write_op_t) -> ();
    pub fn rados_write_op_set_alloc_hint(
        write_op: rados_write_op_t,
        expected_object_size: uint64_t,
        expected_write_size: uint64_t,
    ) -> ();
    pub fn rados_write_op_operate(
        write_op: rados_write_op_t,
        io: rados_ioctx_t,
        oid: *const ::libc::c_char,
        mtime: *mut time_t,
        flags: ::libc::c_int,
    ) -> ::libc::c_int;
    pub fn rados_aio_write_op_operate(
        write_op: rados_write_op_t,
        io: rados_ioctx_t,
        completion: rados_completion_t,
        oid: *const ::libc::c_char,
        mtime: *mut time_t,
        flags: ::libc::c_int,
    ) -> ::libc::c_int;
    pub fn rados_create_read_op() -> rados_read_op_t;
    pub fn rados_release_read_op(read_op: rados_read_op_t) -> ();
    pub fn rados_read_op_set_flags(read_op: rados_read_op_t, flags: ::libc::c_int) -> ();
    pub fn rados_read_op_assert_exists(read_op: rados_read_op_t) -> ();
    pub fn rados_read_op_assert_version(write_op: rados_read_op_t, ver: uint64_t) -> ();
    pub fn rados_read_op_cmpxattr(
        read_op: rados_read_op_t,
        name: *const ::libc::c_char,
        comparison_operator: uint8_t,
        value: *const ::libc::c_char,
        value_len: size_t,
    ) -> ();
    pub fn rados_read_op_getxattrs(
        read_op: rados_read_op_t,
        iter: *mut rados_xattrs_iter_t,
        prval: *mut ::libc::c_int,
    ) -> ();
    pub fn rados_read_op_omap_cmp(
        read_op: rados_read_op_t,
        key: *const ::libc::c_char,
        comparison_operator: uint8_t,
        val: *const ::libc::c_char,
        val_len: size_t,
        prval: *mut ::libc::c_int,
    ) -> ();
    pub fn rados_read_op_stat(
        read_op: rados_read_op_t,
        psize: *mut uint64_t,
        pmtime: *mut time_t,
        prval: *mut ::libc::c_int,
    ) -> ();
    pub fn rados_read_op_read(
        read_op: rados_read_op_t,
        offset: uint64_t,
        len: size_t,
        buf: *mut ::libc::c_char,
        bytes_read: *mut size_t,
        prval: *mut ::libc::c_int,
    ) -> ();
    pub fn rados_read_op_exec(
        read_op: rados_read_op_t,
        cls: *const ::libc::c_char,
        method: *const ::libc::c_char,
        in_buf: *const ::libc::c_char,
        in_len: size_t,
        out_buf: *mut *mut ::libc::c_char,
        out_len: *mut size_t,
        prval: *mut ::libc::c_int,
    ) -> ();
    pub fn rados_read_op_exec_user_buf(
        read_op: rados_read_op_t,
        cls: *const ::libc::c_char,
        method: *const ::libc::c_char,
        in_buf: *const ::libc::c_char,
        in_len: size_t,
        out_buf: *mut ::libc::c_char,
        out_len: size_t,
        used_len: *mut size_t,
        prval: *mut ::libc::c_int,
    ) -> ();
    pub fn rados_read_op_omap_get_vals(
        read_op: rados_read_op_t,
        start_after: *const ::libc::c_char,
        filter_prefix: *const ::libc::c_char,
        max_return: uint64_t,
        iter: *mut rados_omap_iter_t,
        prval: *mut ::libc::c_int,
    ) -> ();
    pub fn rados_read_op_omap_get_keys(
        read_op: rados_read_op_t,
        start_after: *const ::libc::c_char,
        max_return: uint64_t,
        iter: *mut rados_omap_iter_t,
        prval: *mut ::libc::c_int,
    ) -> ();
    pub fn rados_read_op_omap_get_vals_by_keys(
        read_op: rados_read_op_t,
        keys: *const *const ::libc::c_char,
        keys_len: size_t,
        iter: *mut rados_omap_iter_t,
        prval: *mut ::libc::c_int,
    ) -> ();
    pub fn rados_read_op_operate(
        read_op: rados_read_op_t,
        io: rados_ioctx_t,
        oid: *const ::libc::c_char,
        flags: ::libc::c_int,
    ) -> ::libc::c_int;
    pub fn rados_aio_read_op_operate(
        read_op: rados_read_op_t,
        io: rados_ioctx_t,
        completion: rados_completion_t,
        oid: *const ::libc::c_char,
        flags: ::libc::c_int,
    ) -> ::libc::c_int;
    pub fn rados_lock_exclusive(
        io: rados_ioctx_t,
        o: *const ::libc::c_char,
        name: *const ::libc::c_char,
        cookie: *const ::libc::c_char,
        desc: *const ::libc::c_char,
        duration: *mut timeval,
        flags: uint8_t,
    ) -> ::libc::c_int;
    pub fn rados_lock_shared(
        io: rados_ioctx_t,
        o: *const ::libc::c_char,
        name: *const ::libc::c_char,
        cookie: *const ::libc::c_char,
        tag: *const ::libc::c_char,
        desc: *const ::libc::c_char,
        duration: *mut timeval,
        flags: uint8_t,
    ) -> ::libc::c_int;
    pub fn rados_unlock(
        io: rados_ioctx_t,
        o: *const ::libc::c_char,
        name: *const ::libc::c_char,
        cookie: *const ::libc::c_char,
    ) -> ::libc::c_int;
    pub fn rados_list_lockers(
        io: rados_ioctx_t,
        o: *const ::libc::c_char,
        name: *const ::libc::c_char,
        exclusive: *mut ::libc::c_int,
        tag: *mut ::libc::c_char,
        tag_len: *mut size_t,
        clients: *mut ::libc::c_char,
        clients_len: *mut size_t,
        cookies: *mut ::libc::c_char,
        cookies_len: *mut size_t,
        addrs: *mut ::libc::c_char,
        addrs_len: *mut size_t,
    ) -> ssize_t;
    pub fn rados_break_lock(
        io: rados_ioctx_t,
        o: *const ::libc::c_char,
        name: *const ::libc::c_char,
        client: *const ::libc::c_char,
        cookie: *const ::libc::c_char,
    ) -> ::libc::c_int;
    pub fn rados_blacklist_add(
        cluster: rados_t,
        client_address: *mut ::libc::c_char,
        expire_seconds: uint32_t,
    ) -> ::libc::c_int;
    pub fn rados_mon_command(
        cluster: rados_t,
        cmd: *mut *const ::libc::c_char,
        cmdlen: size_t,
        inbuf: *const ::libc::c_char,
        inbuflen: size_t,
        outbuf: *mut *mut ::libc::c_char,
        outbuflen: *mut size_t,
        outs: *mut *mut ::libc::c_char,
        outslen: *mut size_t,
    ) -> ::libc::c_int;
    pub fn rados_mon_command_target(
        cluster: rados_t,
        name: *const ::libc::c_char,
        cmd: *mut *const ::libc::c_char,
        cmdlen: size_t,
        inbuf: *const ::libc::c_char,
        inbuflen: size_t,
        outbuf: *mut *mut ::libc::c_char,
        outbuflen: *mut size_t,
        outs: *mut *mut ::libc::c_char,
        outslen: *mut size_t,
    ) -> ::libc::c_int;
    pub fn rados_buffer_free(buf: *mut ::libc::c_char) -> ();
    pub fn rados_osd_command(
        cluster: rados_t,
        osdid: ::libc::c_int,
        cmd: *mut *const ::libc::c_char,
        cmdlen: size_t,
        inbuf: *const ::libc::c_char,
        inbuflen: size_t,
        outbuf: *mut *mut ::libc::c_char,
        outbuflen: *mut size_t,
        outs: *mut *mut ::libc::c_char,
        outslen: *mut size_t,
    ) -> ::libc::c_int;
    pub fn rados_pg_command(
        cluster: rados_t,
        pgstr: *const ::libc::c_char,
        cmd: *mut *const ::libc::c_char,
        cmdlen: size_t,
        inbuf: *const ::libc::c_char,
        inbuflen: size_t,
        outbuf: *mut *mut ::libc::c_char,
        outbuflen: *mut size_t,
        outs: *mut *mut ::libc::c_char,
        outslen: *mut size_t,
    ) -> ::libc::c_int;
    pub fn rados_monitor_log(
        cluster: rados_t,
        level: *const ::libc::c_char,
        cb: rados_log_callback_t,
        arg: *mut ::std::os::raw::c_void,
    ) -> ::libc::c_int;
}
