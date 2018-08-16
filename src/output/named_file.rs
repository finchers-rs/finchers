use std::cmp;
use std::fs::Metadata;
use std::io;
use std::mem::PinMut;
use std::path::PathBuf;
use std::task::Poll;

use futures_core::future::Future;
use futures_util::compat::{Future01CompatExt, TokioDefaultSpawn};
use futures_util::try_future::TryFutureExt;
use futures_util::try_stream::TryStreamExt;
use futures_util::{future, ready, stream, try_ready};

use tokio::fs::File;
use tokio::prelude::{Async, AsyncRead};

use bytes::{BufMut, BytesMut};
use http::Response;
use mime_guess::guess_mime_type;

use crate::error::Never;
use crate::input::Input;
use crate::output::payload::Body;
use crate::output::Responder;

fn poll_compat<T, E>(input: Result<Async<T>, E>) -> Poll<Result<T, E>> {
    match input {
        Ok(Async::Ready(meta)) => Poll::Ready(Ok(meta)),
        Ok(Async::NotReady) => Poll::Pending,
        Err(err) => Poll::Ready(Err(err)),
    }
}

/// An instance of `Responder` representing a file on the file system.
#[derive(Debug)]
pub struct NamedFile {
    file: File,
    meta: Metadata,
    path: PathBuf,
}

impl NamedFile {
    #[allow(missing_docs)]
    pub fn open(path: PathBuf) -> impl Future<Output = io::Result<NamedFile>> + Send + 'static {
        File::open(path.clone()).compat().and_then(|file| {
            let mut file_opt = Some(file);
            future::poll_fn(move |_| {
                let meta = try_ready!(poll_compat(file_opt.as_mut().unwrap().poll_metadata()));
                Poll::Ready(Ok((file_opt.take().unwrap(), meta)))
            }).map_ok(|(file, meta)| NamedFile { file, meta, path })
        })
    }
}

impl Responder for NamedFile {
    type Body = Body;
    type Error = Never;

    fn respond(self, _: PinMut<'_, Input>) -> Result<Response<Self::Body>, Self::Error> {
        let NamedFile { file, meta, path } = self;

        let buf_size = optimal_buf_size(&meta);
        let len = meta.len();
        let body = file_stream(file, buf_size, len);

        let content_type = guess_mime_type(&path);

        Ok(Response::builder()
            .header("content-length", len)
            .header("content-type", content_type.as_ref())
            .body(body)
            .unwrap())
    }
}

fn optimal_buf_size(meta: &Metadata) -> usize {
    let blk_size = get_block_size(meta);
    cmp::min(blk_size as u64, meta.len()) as usize
}

#[cfg(unix)]
fn get_block_size(meta: &Meta) -> usize {
    use std::os::unix::fs::MetadataExt;
    meta.blksize() as usize()
}

#[cfg(not(unix))]
fn get_block_size(_: &Metadata) -> usize {
    8192
}

fn file_stream(mut file: File, buf_size: usize, mut len: u64) -> Body {
    let mut buf = BytesMut::new();
    let stream = stream::poll_fn(move |_| {
        if len == 0 {
            return Poll::Ready(None);
        }
        if buf.remaining_mut() < buf_size {
            buf.reserve(buf_size);
        }

        let n = match ready!(poll_compat(file.read_buf(&mut buf))) {
            Ok(n) => n as u64,
            Err(e) => return Poll::Ready(Some(Err(e))),
        };
        if n == 0 {
            return Poll::Ready(None);
        }

        let mut chunk = buf.take().freeze();
        if n > len {
            chunk = chunk.split_to(len as usize);
            len = 0;
        } else {
            len = n;
        }
        Poll::Ready(Some(Ok(chunk)))
    });

    Body::wrap_stream(Box::new(stream).compat(TokioDefaultSpawn))
}
