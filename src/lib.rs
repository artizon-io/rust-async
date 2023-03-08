use futures::{
    // FutureExt provides extra methods on the Future trait
    // such as `boxed()`
    future::{BoxFuture, FutureExt},
    task::{waker_ref, ArcWake},
};
use std::{
    future::Future,
    sync::mpsc::{sync_channel, Receiver, SyncSender},
    sync::{Arc, Mutex},
    task::Context,
};
pub use timer_future::TimerFuture;

mod timer_future;

/// A simple task executor
///
/// Receives tasks (top-level `Future`s) off of a channel and runs them.
pub struct Executor {
    receiver: Receiver<Arc<Task>>,
}

impl Executor {
    pub fn run(&self) {
        // Keep polling `Future` until no tasks received
        // In this case, sending a task through the
        // channel means it is "waken"
        while let Ok(task) = self.receiver.recv() {
            let mut future_slot = task.future.lock().unwrap();
            if let Some(mut future) = future_slot.take() {
                // Creates a `Waker` using a `Arc<impl ArcWake>`
                let waker = waker_ref(&task);
                // Since context currently is only used to store a reference to waker,
                // we can create a context directly using a waker
                let context = &mut Context::from_waker(&*waker);
                // `Pin::as_mut()`: convert Box to &mut
                if future.as_mut().poll(context).is_pending() {
                    // After poll, `Future` is still pending,
                    // hence we put it back into the `Option`
                    *future_slot = Some(future);
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct TaskSpawner {
    sender: SyncSender<Arc<Task>>,
}

impl TaskSpawner {
    // Another way to write generics
    pub fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
        // Turn `Future` into `BoxFuture` (from futures crate)
        // Same as wrapping inside Box and pinning it
        let future = future.boxed();

        // Create a shared smart pointer to `Task`
        let task = Arc::new(Task {
            future: Mutex::new(Some(future)),
            // Safe to clone the sender because it is mpsc
            sender: self.sender.clone(),
        });
        // The "initial" send event (without "wake")
        // Latter send events will be issued when "wake()" is called
        self.sender
            .send(task)
            .expect("Number of tasks exceed MAX_QUEUED_TASKS");
    }
}

/// A `Future` that can send itself through the channel
struct Task {
    // Compiler doesn't know that `Future` will ever only be mutated from one thread,
    // hence we "mutex" it
    future: Mutex<Option<BoxFuture<'static, ()>>>,
    sender: SyncSender<Arc<Task>>,
}

// See the `ArcWake` trait in `futures`
// https://rust-lang-nursery.github.io/futures-api-docs/0.3.0-alpha.13/futures/task/trait.ArcWake.html
impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        // Forcing first argument (self) to be a `Arc` smart pointer
        let cloned = arc_self.clone();
        // "Wake" by just sending a task down the channel
        arc_self
            .sender
            .send(cloned)
            .expect("Number of tasks exceed MAX_QUEUED_TASKS");
    }
}

pub fn new_executor_and_spawner() -> (Executor, TaskSpawner) {
    const MAX_QUEUED_TASKS: usize = 10_000;
    let (sender, receiver) = sync_channel(MAX_QUEUED_TASKS);
    (Executor { receiver }, TaskSpawner { sender })
}
