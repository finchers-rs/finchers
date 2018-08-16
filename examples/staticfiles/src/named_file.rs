use std::cmp;
use std::fs::Metadata;
use std::io;
use std::mem::PinMut;
use std::path::PathBuf;
use std::task::Poll;

use futures::compat::{Future01CompatExt, TokioDefaultSpawn};
use futures::stream::TryStreamExt;
use futures::{future, ready, stream};

use tokio::fs::File;
use tokio::prelude::{Async, AsyncRead};

use bytes::{BufMut, BytesMut};
use http::Response;
use mime_guess::guess_mime_type;

use finchers::error::Never;
use finchers::input::Input;
use finchers::output::payload::Body;
use finchers::output::Responder;

#[derive(Debug)]
pub struct NamedFile {
    file: File,
    meta: Metadata,
    path: PathBuf,
}

impl NamedFile {
    pub async fn open(path: PathBuf) -> io::Result<NamedFile> {
        let mut file = await!(File::open(path.clone()).compat())?;
        let meta = await!(future::poll_fn(|_| poll_compat(file.poll_metadata())))?;
        Ok(NamedFile { file, meta, path })
    }
}

impl Responder for NamedFile {
    type Body = Body;
    type Error = Never;

    fn respond(self, _: PinMut<Input>) -> Result<Response<Self::Body>, Self::Error> {
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

fn poll_compat<T, E>(input: Result<Async<T>, E>) -> Poll<Result<T, E>> {
    match input {
        Ok(Async::Ready(meta)) => Poll::Ready(Ok(meta)),
        Ok(Async::NotReady) => Poll::Pending,
        Err(err) => Poll::Ready(Err(err)),
    }
}
