use std::ffi::CStr;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::executor::ThreadPool;
use futures::task::SpawnExt;
use futures::{Future, Stream};

use crate::ceph::CephObject;
use crate::error::{RadosError, RadosResult};
use crate::rados::{rados_list_ctx_t, rados_nobjects_list_close, rados_nobjects_list_next};

/// Wrap rados_list_ctx_t to make it Send (hold across .await)
#[derive(Copy, Clone)]
struct ListCtxHandle(rados_list_ctx_t);
unsafe impl Send for ListCtxHandle {}

/// A high level Stream interface to the librados 'nobjects_list' functionality.
///
/// librados does not expose asynchronous calls for object listing, so we use
/// a background helper thread.
pub struct ListStream {
    ctx: ListCtxHandle,
    workers: ThreadPool,

    // We only have a single call to nobjects_list_next outstanding at
    // any time: rely on underlying librados/Objecter to do
    // batching/readahead
    next: Option<Pin<Box<dyn Future<Output = Option<RadosResult<CephObject>>>>>>,
}

unsafe impl Send for ListStream {}

impl ListStream {
    pub fn new(ctx: rados_list_ctx_t) -> Self {
        Self {
            ctx: ListCtxHandle(ctx),
            workers: ThreadPool::builder()
                .pool_size(1)
                .create()
                .expect("Could not spawn worker thread"),
            next: None,
        }
    }
}

impl Stream for ListStream {
    type Item = Result<CephObject, RadosError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.next.is_none() {
            let list_ctx = self.ctx;
            self.next = Some(Box::pin(
                self.workers
                    .spawn_with_handle(async move {
                        let mut entry_ptr: *mut *const ::libc::c_char = std::ptr::null_mut();
                        let mut key_ptr: *mut *const ::libc::c_char = std::ptr::null_mut();
                        let mut nspace_ptr: *mut *const ::libc::c_char = std::ptr::null_mut();
                        unsafe {
                            let r = rados_nobjects_list_next(
                                list_ctx.0,
                                &mut entry_ptr,
                                &mut key_ptr,
                                &mut nspace_ptr,
                            );

                            if r == -libc::ENOENT {
                                None
                            } else if r < 0 {
                                Some(Err(r.into()))
                            } else {
                                let object_name =
                                    CStr::from_ptr(entry_ptr as *const ::libc::c_char);
                                let mut object_locator = String::new();
                                let mut namespace = String::new();
                                if !key_ptr.is_null() {
                                    object_locator.push_str(
                                        &CStr::from_ptr(key_ptr as *const ::libc::c_char)
                                            .to_string_lossy(),
                                    );
                                }
                                if !nspace_ptr.is_null() {
                                    namespace.push_str(
                                        &CStr::from_ptr(nspace_ptr as *const ::libc::c_char)
                                            .to_string_lossy(),
                                    );
                                }

                                Some(Ok(CephObject {
                                    name: object_name.to_string_lossy().into_owned(),
                                    entry_locator: object_locator,
                                    namespace,
                                }))
                            }
                        }
                    })
                    .expect("Could not spawn background task"),
            ));
        }

        let result = self.next.as_mut().unwrap().as_mut().poll(cx);
        match &result {
            Poll::Pending => Poll::Pending,
            _ => {
                self.next = None;
                result
            }
        }

        // match self.next.as_mut().unwrap().as_mut().poll(cx) {
        //     Poll::Pending => Poll: Pending,
        //     Poll::Ready(None) => Poll::Ready(None),
        //     Poll::Ready(Some(Err(rados_error))) => Poll::Ready(Some(Err(rados_error))),
        //     Poll::Ready(Some(Ok(ceph_object))) => Poll::Ready(Some(Err(rados_error))),
        // }
    }
}

impl Drop for ListStream {
    fn drop(&mut self) {
        unsafe {
            rados_nobjects_list_close(self.ctx.0);
        }
    }
}
