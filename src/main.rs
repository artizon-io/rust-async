use rust_async::{new_executor_and_spawner, AndThenFuture, JoinFuture, TimerFuture};
use std::time::Duration;

fn main() {
    let (executor, spawner) = new_executor_and_spawner();

    let one_second_future = async {
        println!("Before 1 seconds countdown");
        TimerFuture::new(Duration::new(1, 0)).await;
        println!("After 1 seconds countdown");
    };

    let two_second_future = async {
        println!("Before 2 seconds countdown");
        TimerFuture::new(Duration::new(2, 0)).await;
        println!("After 2 seconds countdown");
    };

    let five_second_future = async {
        println!("Before 5 seconds countdown");
        TimerFuture::new(Duration::new(5, 0)).await;
        println!("After 5 seconds countdown");
    };

    let join_future = JoinFuture::new(five_second_future, two_second_future);
    let and_then_future = AndThenFuture::new(join_future, one_second_future);

    spawner.spawn(and_then_future);

    // spawner.spawn(one_second_future);
    // spawner.spawn(two_second_future);
    // spawner.spawn(five_second_future);

    // Drop the spawner so that our executor knows that there
    // won't be more incoming tasks
    drop(spawner);

    // Run the executor until the task queue is empty
    executor.run();
}
