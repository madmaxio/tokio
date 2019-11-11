use std::pin::Pin;
use std::task::{Context, Poll};
use std::future::Future;

use pin_project::pin_project;
use futures::{ready, TryFuture};

use super::{Filter, FilterBase, Tuple};

#[derive(Clone, Copy, Debug)]
pub struct UntupleOne<F> {
    pub(super) filter: F,
}

impl<F, T> FilterBase for UntupleOne<F>
where
    F: Filter<Extract = (T,)>,
    T: Tuple,
{
    type Extract = T;
    type Error = F::Error;
    type Future = UntupleOneFuture<F>;
    #[inline]
    fn filter(&self) -> Self::Future {
        UntupleOneFuture {
            extract: self.filter.filter(),
        }
    }
}

#[allow(missing_debug_implementations)]
#[pin_project]
pub struct UntupleOneFuture<F: Filter> {
    #[pin]
    extract: F::Future,
}

impl<F, T> Future for UntupleOneFuture<F>
where
    F: Filter<Extract = (T,)>,
    T: Tuple,
{
    type Output = Result<T, F::Error>;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        match ready!(self.project().extract.try_poll(cx)) {
            Ok((t,)) => Poll::Ready(Ok(t)),
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}
