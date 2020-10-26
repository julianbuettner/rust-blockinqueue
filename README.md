# BlockingQueue
A very very simple wrapper around Rust's mspc channel
to work as a blocking queue.

## Usage
Here is a little example on how to use it:

```rust
use blockingqueue::BlockingQueue;
use std::{thread, time};

fn main() {
    let bq = BlockingQueue::new();

    let bq_clone1 = bq.clone();
    thread::spawn(move || {
        thread::sleep(time::Duration::from_millis(100));
        bq_clone1.push(123);
        bq_clone1.push(456);
        bq_clone1.push(789);
    });

    let bq_clone2 = bq.clone();
    thread::spawn(move || {
        thread::sleep(time::Duration::from_millis(400));
        bq_clone2.push(321);
        bq_clone2.push(654);
        bq_clone2.push(987);
    });

    let bq_clone3 = bq.clone();
    let read_three_thread = thread::spawn(move || {
        for _ in 0..3 {
            println!("Popped in child thread: {}", bq_clone3.pop());
        }
    });

    for _ in 0..3 {
        println!("Popped in parent thread: {}", bq.pop());
    }

    read_three_thread.join().unwrap();

    println!("I will wait forever here...");
    println!("{}", bq.pop());
}

```

