#![allow(missing_docs)]

use std::mem::PinMut;
use std::string::FromUtf8Error;
use std::task::{self, Poll};

use bytes::Bytes;
use failure::Fail;
use futures::{self as futures01, Async};
use http::header::HeaderMap;
use hyper::body::{Body, Chunk, Payload as _Payload};
use pin_utils::unsafe_unpinned;

use crate::error::{fail, Error, Never};
use crate::input::Input;

#[derive(Debug)]
pub struct Payload {
    body: Body,
}

impl Payload {
    unsafe_unpinned!(body: Body);

    pub fn poll_data(
        mut self: PinMut<'_, Self>,
        cx: &mut task::Context<'_>,
    ) -> Poll<Result<Option<Chunk>, Error>> {
        poll_01_with_cx(cx, || self.body().poll_data()).map_err(fail)
    }

    pub fn poll_trailers(
        mut self: PinMut<'_, Self>,
        cx: &mut task::Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Error>> {
        poll_01_with_cx(cx, || self.body().poll_trailers()).map_err(fail)
    }

    pub fn is_end_stream(&self) -> bool {
        self.body.is_end_stream()
    }

    pub fn content_length(&self) -> Option<u64> {
        self.body.content_length()
    }
}

/// An asyncrhonous stream to receive the chunks of incoming request body.
#[derive(Debug)]
pub struct ReqBody(Option<Body>);

impl ReqBody {
    /// Create an instance of `RequestBody` from `hyper::Body`.
    pub fn from_hyp(body: Body) -> ReqBody {
        ReqBody(Some(body))
    }

    #[allow(missing_docs)]
    pub fn payload(&mut self) -> Option<Payload> {
        self.0.take().map(|body| Payload { body })
    }

    pub fn is_gone(&self) -> bool {
        self.0.is_none()
    }
}

/// Trait representing the transformation from a message body.
pub trait FromBody: 'static + Sized {
    /// The error type which will be returned from `from_data`.
    type Error: Fail;

    /// Performs conversion from raw bytes into itself.
    fn from_body(body: Bytes, input: PinMut<'_, Input>) -> Result<Self, Self::Error>;
}

impl FromBody for Bytes {
    type Error = Never;

    fn from_body(body: Bytes, _: PinMut<'_, Input>) -> Result<Self, Self::Error> {
        Ok(body)
    }
}

impl FromBody for String {
    type Error = FromUtf8Error;

    fn from_body(body: Bytes, _: PinMut<'_, Input>) -> Result<Self, Self::Error> {
        String::from_utf8(body.to_vec())
    }
}

// ==== compat ====

fn poll_01_with_cx<T, E>(
    cx: &mut task::Context<'_>,
    f: impl FnOnce() -> futures01::Poll<T, E>,
) -> Poll<Result<T, E>> {
    // FIXME: Set the executor to the global context for futures-0.1
    let notify = &WakerToHandle(cx.waker());

    futures01::executor::with_notify(notify, 0, move || match f() {
        Ok(Async::Ready(ok)) => Poll::Ready(Ok(ok)),
        Ok(Async::NotReady) => Poll::Pending,
        Err(err) => Poll::Ready(Err(err)),
    })
}

#[allow(missing_debug_implementations)]
struct NotifyWaker(task::Waker);

#[derive(Clone)]
#[allow(missing_debug_implementations)]
struct WakerToHandle<'a>(&'a task::Waker);

impl<'a> From<WakerToHandle<'a>> for futures01::executor::NotifyHandle {
    fn from(handle: WakerToHandle<'a>) -> futures01::executor::NotifyHandle {
        let ptr = Box::new(NotifyWaker(handle.0.clone()));
        unsafe { futures01::executor::NotifyHandle::new(Box::into_raw(ptr)) }
    }
}

impl futures01::executor::Notify for NotifyWaker {
    fn notify(&self, _: usize) {
        self.0.wake()
    }
}

unsafe impl futures01::executor::UnsafeNotify for NotifyWaker {
    unsafe fn clone_raw(&self) -> futures01::executor::NotifyHandle {
        WakerToHandle(&self.0).into()
    }

    unsafe fn drop_raw(&self) {
        let ptr: *const dyn futures01::executor::UnsafeNotify = self;
        drop(Box::from_raw(
            ptr as *mut dyn futures01::executor::UnsafeNotify,
        ));
    }
}
