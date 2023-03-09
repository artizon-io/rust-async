use futures::future::BoxFuture;
use futures::FutureExt;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// A Future that runs two futures sequentially
///
/// Simple clone of the `AndThen` combinator.
// `AndThen` combinator allows creating the second future based on the output
// of the first future, like `get_breakfast.and_then(|food| eat(food))`.
pub struct AndThenFuture {
    first: Option<BoxFuture<'static, ()>>,
    second: BoxFuture<'static, ()>,
}

impl AndThenFuture {
    pub fn new(
        first: impl Future<Output = ()> + 'static + Send,
        second: impl Future<Output = ()> + 'static + Send,
    ) -> Self {
        AndThenFuture {
            first: Some(first.boxed()),
            second: second.boxed(),
        }
    }
}

impl Future for AndThenFuture {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(mut first) = self.first.take() {
            match first.as_mut().poll(cx) {
                Poll::Ready(()) => (),
                Poll::Pending => {
                    self.first = Some(first);
                    return Poll::Pending;
                }
            };
        }
        // Now that the first future is done, attempt to complete the second.
        // Modifying the second future in place
        self.second.as_mut().poll(cx)
    }
}
