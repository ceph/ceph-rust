// Copyright 2021 John Spray All rights reserved.
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

use std::ffi::c_void;
use std::pin::Pin;
use std::sync::Mutex;
use std::task::{Context, Poll, Waker};

use crate::ceph::IoCtx;
use crate::error::RadosResult;
use crate::rados::{
    rados_aio_cancel, rados_aio_create_completion2, rados_aio_get_return_value,
    rados_aio_is_complete, rados_aio_release, rados_aio_wait_for_complete_and_cb,
    rados_completion_t,
};

pub(crate) struct Completion<'a> {
    inner: rados_completion_t,

    // Box to provide a stable address for completion_complete callback
    // Mutex for memory fencing when writing from poll() and reading from completion_complete()
    waker: Box<std::sync::Mutex<Option<std::task::Waker>>>,

    // A reference to the IOCtx is required to issue a cancel on
    // the operation if we are dropped before ready.  This needs
    // to be a Rust reference rather than a raw rados_ioctx_t because otherwise
    // there would be nothing to stop the rados_ioctx_t being invalidated
    // during the lifetime of this Completion.
    // (AioCompletionImpl does hold a reference to IoCtxImpl for writes, but
    //  not for reads.)
    ioctx: &'a IoCtx,
}

unsafe impl Send for Completion<'_> {}

#[no_mangle]
pub extern "C" fn completion_complete(_cb: rados_completion_t, arg: *mut c_void) -> () {
    let waker = unsafe {
        let p = arg as *mut Mutex<Option<Waker>>;
        p.as_mut().unwrap()
    };

    let waker = waker.lock().unwrap().take();
    match waker {
        Some(w) => w.wake(),
        None => {}
    }
}

impl Drop for Completion<'_> {
    fn drop(&mut self) {
        // Ensure that after dropping the Completion, the AIO callback
        // will not be called on our dropped waker Box.  Only necessary
        // if we got as far as successfully starting an operation using
        // the completion.
        let am_complete = unsafe { rados_aio_is_complete(self.inner) } != 0;
        if !am_complete {
            unsafe {
                let cancel_r = rados_aio_cancel(self.ioctx.ioctx, self.inner);

                // It is unsound to proceed if the Objecter op is still in flight
                assert!(cancel_r == 0 || cancel_r == -libc::ENOENT);
            }
        }

        unsafe {
            // Even if is_complete was true, librados might not be done with
            // our callback: wait til it is.
            assert_eq!(rados_aio_wait_for_complete_and_cb(self.inner), 0);
        }

        unsafe {
            rados_aio_release(self.inner);
        }
    }
}

impl std::future::Future for Completion<'_> {
    type Output = crate::error::RadosResult<u32>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Hold lock across the check of am_complete and subsequent waker registration
        // to avoid deadlock if callback is invoked in between.
        let mut waker_locked = self.waker.lock().unwrap();

        let am_complete = unsafe { rados_aio_is_complete(self.inner) } != 0;

        if am_complete {
            // Unlock Waker so that completion callback can complete if racing with us.
            drop(waker_locked);

            // Ensure librados is finished with our callback ('complete' is true
            // before it calls that)
            unsafe {
                let r = rados_aio_wait_for_complete_and_cb(self.inner);
                assert_eq!(r, 0);
            }

            let r = unsafe { rados_aio_get_return_value(self.inner) };
            let result = if r < 0 { Err(r.into()) } else { Ok(r) };
            std::task::Poll::Ready(result.map(|e| e as u32))
        } else {
            // Register a waker
            *waker_locked = Some(cx.waker().clone());

            std::task::Poll::Pending
        }
    }
}

/// Completions are only created via this wrapper, in order to ensure
/// that the Completion struct is only constructed around 'armed' rados_completion_t
/// instances (i.e. those that have been used to start an I/O).
pub(crate) fn with_completion<F>(ioctx: &IoCtx, f: F) -> RadosResult<Completion<'_>>
where
    F: FnOnce(rados_completion_t) -> libc::c_int,
{
    let mut waker = Box::new(Mutex::new(None));

    let completion = unsafe {
        let mut completion: rados_completion_t = std::ptr::null_mut();
        let p: *mut Mutex<Option<Waker>> = &mut *waker;
        let p = p as *mut c_void;

        let r = rados_aio_create_completion2(p, Some(completion_complete), &mut completion);
        if r != 0 {
            panic!("Error {} allocating RADOS completion: out of memory?", r);
        }
        assert!(!completion.is_null());

        completion
    };

    let ret_code = f(completion);

    if ret_code < 0 {
        // On error dispatching I/O, drop the unused rados_completion_t
        unsafe {
            rados_aio_release(completion);
            drop(completion)
        }
        Err(ret_code.into())
    } else {
        // Pass the rados_completion_t into a Future-implementing wrapper and await it.
        Ok(Completion {
            ioctx,
            inner: completion,
            waker,
        })
    }
}
