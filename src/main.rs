use rust_async::{new_executor_and_spawner, TimerFuture};
use std::time::Duration;

fn main() {
    let (executor, spawner) = new_executor_and_spawner();

    spawner.spawn(async {
        println!("Before 1 seconds countdown");
        TimerFuture::new(Duration::new(1, 0)).await;
        println!("After 1 seconds countdown");
    });

    spawner.spawn(async {
        println!("Before 2 seconds countdown");
        TimerFuture::new(Duration::new(2, 0)).await;
        println!("After 2 seconds countdown");
    });

    spawner.spawn(async {
        println!("Before 5 seconds countdown");
        TimerFuture::new(Duration::new(5, 0)).await;
        println!("After 5 seconds countdown");
    });

    // Drop the spawner so that our executor knows that there
    // won't be more incoming tasks
    drop(spawner);

    // Run the executor until the task queue is empty
    executor.run();
}
