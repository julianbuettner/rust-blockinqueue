use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};

pub struct BlockingQueue<T> {
    sender: Sender<T>,
    receiver: Arc<Mutex<Receiver<T>>>,
}

impl<T> BlockingQueue<T> {
    pub fn new() -> Self {
        let (sender, receiver) = channel();
        Self {
            sender: sender,
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }

    pub fn push(&self, e: T) {
        self.sender.send(e).unwrap();
    }

    pub fn pop(&self) -> T {
        self.receiver.lock().unwrap().recv().unwrap()
    }
}

impl<T> Clone for BlockingQueue<T> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            receiver: self.receiver.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{thread, time};

    #[test]
    fn test_1() {
        let bq = BlockingQueue::new();
        bq.push(123);
        bq.push(456);
        bq.push(789);
        assert_eq!(bq.pop(), 123);
        assert_eq!(bq.pop(), 456);
        assert_eq!(bq.pop(), 789);
    }

    #[test]
    fn test_2() {
        let bq = BlockingQueue::new();

        let bq0 = bq.clone();
        thread::spawn(move || {
            thread::sleep(time::Duration::from_millis(100));
            bq0.push(123);
        });

        let bq1 = bq.clone();
        thread::spawn(move || {
            thread::sleep(time::Duration::from_millis(1000));
            bq1.push(456);
        });

        assert_eq!(bq.pop(), 123);
        assert_eq!(bq.pop(), 456);
    }
}
