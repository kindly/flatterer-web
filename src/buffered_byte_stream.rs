use async_std::io::{self, Read};
use async_std::stream::Stream;
use async_std::task::{Context, Poll};

use std::pin::Pin;

#[derive(Debug)]
pub struct BufferedBytesStream<T> {
    pub inner: T,
}

impl<T: Read + Unpin> Stream for BufferedBytesStream<T> {
    //type Item = async_std::io::Result<Vec<u8>>;
    type Item = async_std::io::Result<Vec<u8>>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut buf = [0u8; 2048];

        let rd = Pin::new(&mut self.inner);

        match futures_core::ready!(rd.poll_read(cx, &mut buf)) {
            Ok(0) => Poll::Ready(None),
            Ok(n) => Poll::Ready(Some(Ok(buf[..n].to_vec()))),
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => Poll::Pending,
            Err(e) => Poll::Ready(Some(Err(e))),
        }
    }
}
