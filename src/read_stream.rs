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
// limitations under the License

use futures::{FutureExt, Stream};
use std::ffi::CString;
use std::future::Future;
use std::os::raw::c_char;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::ceph::IoCtx;
use crate::completion::with_completion;
use crate::error::RadosResult;
use crate::rados::rados_aio_read;

const DEFAULT_BUFFER_SIZE: usize = 4 * 1024 * 1024;
const DEFAULT_CONCURRENCY: usize = 2;

pub struct ReadStream<'a> {
    ioctx: &'a IoCtx,

    // Size of each RADOS read op
    buffer_size: usize,

    // Number of concurrent RADOS read ops to issue
    concurrency: usize,

    in_flight: Vec<IOSlot<'a>>,

    next: u64,

    object_name: String,

    // Flag is set when we see a short read - means do not issue any more IOs,
    // and return Poll::Ready(None) on next poll
    done: bool,
}

unsafe impl Send for ReadStream<'_> {}

impl<'a> ReadStream<'a> {
    pub(crate) fn new(
        ioctx: &'a IoCtx,
        object_name: &str,
        buffer_size: Option<usize>,
        concurrency: Option<usize>,
    ) -> Self {
        let mut inst = Self {
            ioctx,
            buffer_size: buffer_size.unwrap_or(DEFAULT_BUFFER_SIZE),
            concurrency: concurrency.unwrap_or(DEFAULT_CONCURRENCY),
            in_flight: Vec::new(),
            next: 0,
            object_name: object_name.to_string(),
            done: false,
        };

        // Start IOs early, don't wait for the first poll.
        inst.maybe_issue();

        inst
    }
}

enum IOSlot<'a> {
    Pending(Pin<Box<dyn Future<Output = (Vec<u8>, RadosResult<u32>)> + 'a>>),
    Complete((Vec<u8>, RadosResult<u32>)),
}

impl<'a> ReadStream<'a> {
    fn maybe_issue(&mut self) {
        // Issue reads
        while self.in_flight.len() < self.concurrency {
            let read_at = self.next;
            self.next += self.buffer_size as u64;

            // Inefficient: copying out string to dodge ownership issues for the moment
            let object_name_bg = self.object_name.clone();

            // Grab items for use inside async{} block to avoid referencing self from in there.
            let ioctx = self.ioctx;
            let read_size = self.buffer_size;

            // Use an async block to tie together the lifetime of a Vec and the Completion that uses it
            let fut = async move {
                let obj_name_str = CString::new(object_name_bg).expect("CString error");
                let mut fill_buffer = Vec::with_capacity(read_size);
                let completion = with_completion(ioctx, |c| unsafe {
                    rados_aio_read(
                        ioctx.ioctx,
                        obj_name_str.as_ptr(),
                        c,
                        fill_buffer.as_mut_ptr() as *mut c_char,
                        fill_buffer.capacity(),
                        read_at,
                    )
                })
                .expect("Can't issue read");

                let result = completion.await;
                if let Ok(rval) = &result {
                    unsafe {
                        let len = *rval as usize;
                        assert!(len <= fill_buffer.capacity());
                        fill_buffer.set_len(len);
                    }
                }

                (fill_buffer, result)
            };

            let mut fut = Box::pin(fut);

            let slot = match fut.as_mut().now_or_never() {
                Some(result) => IOSlot::Complete(result),
                None => IOSlot::Pending(fut),
            };

            self.in_flight.push(slot);
        }
    }
}

impl<'a> Stream for ReadStream<'a> {
    type Item = RadosResult<Vec<u8>>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.done {
            // Our last read result was a short one: we know nothing else needs doing.
            return Poll::Ready(None);
        }

        self.maybe_issue();

        // Poll next read: maybe return pending if none is ready
        let next_op = &mut self.in_flight[0];
        let (buffer, result) = match next_op {
            IOSlot::Complete(_) => {
                let complete = self.in_flight.remove(0);
                if let IOSlot::Complete(c) = complete {
                    c
                } else {
                    panic!("Cannot happen")
                }
            }
            IOSlot::Pending(fut) => match fut.as_mut().poll(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(r) => {
                    self.in_flight.remove(0);
                    r
                }
            },
        };

        self.maybe_issue();

        // A result is ready, handle it.
        match result {
            Ok(length) => {
                if (length as usize) < self.buffer_size {
                    // Cancel outstanding ops
                    self.in_flight.clear();

                    // Flag to return Ready(None) on next call to poll.
                    self.done = true;
                }
                Poll::Ready(Some(Ok(buffer)))
            }
            Err(e) => Poll::Ready(Some(Err(e))),
        }
    }
}
