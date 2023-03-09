use futures::future::BoxFuture;
use futures::FutureExt;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// A Future that runs two futures concurrently
///
/// An allocation-free state machine
///
/// A combinator
pub struct JoinFuture {
    // Each field may contain a future that should be run to completion.
    // If the future has already completed, the field is set to `None`.
    // This prevents us from polling a future after it has completed, which
    // would violate the contract of the `Future` trait.
    a: Option<BoxFuture<'static, ()>>,
    b: Option<BoxFuture<'static, ()>>,
}

impl JoinFuture {
    pub fn new(
        a: impl Future<Output = ()> + 'static + Send,
        b: impl Future<Output = ()> + 'static + Send,
    ) -> Self {
        JoinFuture {
            a: Some(a.boxed()),
            b: Some(b.boxed()),
        }
    }
}

impl Future for JoinFuture {
    type Output = ();
    // We want to mutate self (modify self's fields)
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // `a` needs to be mutable because `as_mut()` borrows a as mutable (in order to return a pinned mutable reference)
        // The mutable ref is used to call `poll`, which may modify the future
        if let Some(mut a) = self.a.take() {
            // Take a out in order to construct a Pin<&mut a>
            // If future is not None (not completed yet),
            // poll it. If result is Ready, Option::take()
            if a.as_mut().poll(cx).is_pending() {
                self.a = Some(a); // Put a back if result is Poll::Pending
            }
        }

        if let Some(mut b) = self.b.take() {
            if b.as_mut().poll(cx).is_pending() {
                self.b = Some(b);
            }
        }

        if self.a.is_none() && self.b.is_none() {
            // Both futures have completed -- we can return successfully
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}
