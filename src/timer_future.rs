use futures::Future;
use std::{
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
    thread,
    time::Duration,
};

/// A simple TimerFuture struct that resolves to Poll::Ready
/// when time elapsed
pub struct TimerFuture {
    shared_state: Arc<Mutex<SharedState>>,
}

struct SharedState {
    completed: bool,
    waker: Option<Waker>,
}

impl TimerFuture {
    pub fn new(duration: Duration) -> Self {
        let shared_state = Arc::new(Mutex::new(SharedState {
            completed: false,
            waker: None,
        }));

        // Clone the shared_state and move it into thread
        let thread_shared_state = shared_state.clone();
        thread::spawn(move || {
            thread::sleep(duration);
            let mut shared_state = thread_shared_state.lock().unwrap();
            // Wake up the last task which the future was polled, if one exists
            shared_state.completed = true;
            if let Some(waker) = shared_state.waker.take() {
                waker.wake() // Invoke waker (of context)
            }
        });

        TimerFuture { shared_state }
    }
}

impl Future for TimerFuture {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut shared_state = self.shared_state.lock().unwrap();
        if shared_state.completed {
            Poll::Ready(())
        } else {
            // `TimerFuture` can move between tasks on the executor
            // and thus `Waker` might keep changing.
            // For now we assume that when `Future` moves to a new task, `poll` will be run immediately.
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}
