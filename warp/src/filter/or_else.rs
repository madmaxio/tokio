use std::pin::Pin;
use std::task::{Context, Poll};
use std::future::Future;

use pin_project::{pin_project, project};
use futures::{ready, TryFuture};

use super::{Filter, FilterBase, Func};
use crate::route;

#[derive(Clone, Copy, Debug)]
pub struct OrElse<T, F> {
    pub(super) filter: T,
    pub(super) callback: F,
}

impl<T, F> FilterBase for OrElse<T, F>
where
    T: Filter,
    F: Func<T::Error> + Clone + Send,
    F::Output: TryFuture<Ok =  T::Extract, Error = T::Error> + Send,
{
    type Extract = <F::Output as TryFuture>::Ok;
    type Error = <F::Output as TryFuture>::Error;
    type Future = OrElseFuture<T, F>;
    #[inline]
    fn filter(&self) -> Self::Future {
        let idx = route::with(|route| route.matched_path_index());
        OrElseFuture {
            state: State::First(self.filter.filter(), self.callback.clone()),
            original_path_index: PathIndex(idx),
        }
    }
}

#[allow(missing_debug_implementations)]
#[pin_project]
pub struct OrElseFuture<T: Filter, F>
where
    T: Filter,
    F: Func<T::Error>,
    F::Output: TryFuture<Ok =  T::Extract, Error = T::Error> + Send,
{
    #[pin]
    state: State<T, F>,
    original_path_index: PathIndex,
}

#[pin_project]
enum State<T, F>
where
    T: Filter,
    F: Func<T::Error>,
    F::Output: TryFuture<Ok =  T::Extract, Error = T::Error> + Send,
{
    First(#[pin] T::Future, F),
    Second(#[pin] F::Output),
    Done,
}

#[derive(Copy, Clone)]
struct PathIndex(usize);

impl PathIndex {
    fn reset_path(&self) {
        route::with(|route| route.reset_matched_path_index(self.0));
    }
}

impl<T, F> Future for OrElseFuture<T, F>
where
    T: Filter,
    F: Func<T::Error>,
    F::Output: TryFuture<Ok =  T::Extract, Error = T::Error> + Send,
{
    type Output = Result<<F::Output as TryFuture>::Ok, <F::Output as TryFuture>::Error>;

    #[project]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        loop {
            let pin = self.as_mut().project();
            #[project]
            let (err, second) = match pin.state.project() {
                State::First(first, second) => match ready!(first.try_poll(cx)) {
                    Ok(ex) => return Poll::Ready(Ok(ex)),
                    Err(err) => (err, second),
                },
                State::Second(second) => {
                    let ex2 = ready!(second.try_poll(cx));
                    self.set(OrElseFuture{ state: State::Done, ..*self });
                    return Poll::Ready(ex2);
                }
                State::Done => panic!("polled after complete"),
            };

            pin.original_path_index.reset_path();
            let fut2 = second.call(err);
            self.set(OrElseFuture{ state: State::Second(fut2), ..*self });
        }
    }
}
