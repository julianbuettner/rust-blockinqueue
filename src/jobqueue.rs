use super::BlockingQueue;
use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex};

pub struct JobHandler<'a, J, R> {
    pub job: J,
    index: usize,
    conditional_result_table: &'a Arc<(Condvar, Mutex<HashMap<usize, R>>)>,
}

impl<'a, J, R> JobHandler<'a, J, R> {
    pub fn commit(&self, result: R) {
        let (cvar, lock) = &**self.conditional_result_table;
        {
            let mut table = lock.lock().unwrap();
            table.insert(self.index, result);
        }
        cvar.notify_all();
    }
}

pub struct StableJobResultQueue<J, R> {
    job_queue: BlockingQueue<(usize, J)>,
    conditional_result_table: Arc<(Condvar, Mutex<HashMap<usize, R>>)>,

    counter_push: Arc<Mutex<usize>>,
    counter_pop: Arc<Mutex<usize>>,
}

impl<J, R> Clone for StableJobResultQueue<J, R> {
    fn clone(&self) -> Self {
        Self {
            job_queue: self.job_queue.clone(),
            conditional_result_table: self.conditional_result_table.clone(),
            counter_pop: self.counter_pop.clone(),
            counter_push: self.counter_push.clone(),
        }
    }
}

impl<J, R> StableJobResultQueue<J, R> {
    pub fn new() -> Self {
        Self {
            job_queue: BlockingQueue::new(),
            conditional_result_table: Arc::new((Condvar::new(), Mutex::new(HashMap::new()))),
            counter_push: Arc::new(Mutex::new(0)),
            counter_pop: Arc::new(Mutex::new(0)),
        }
    }

    pub fn push_job(&self, j: J) {
        let counter_push = {
            let mut v = self.counter_push.lock().unwrap();
            *v += 1;
            *v
        };
        self.job_queue.push((counter_push, j));
    }

    pub fn pop_jobhandler(&self) -> JobHandler<J, R> {
        let (identifier, job) = self.job_queue.pop();
        JobHandler {
            job: job,
            index: identifier,
            conditional_result_table: &self.conditional_result_table,
        }
    }

    pub fn pop_result(&self) -> R {
        let counter_pop = {
            let mut v = self.counter_pop.lock().unwrap();
            *v += 1;
            *v
        };
        let (cvar, lock) = &*self.conditional_result_table;

        let mut table = lock.lock().unwrap();
        match table.remove(&counter_pop) {
            Some(v) => return v,
            None => (),
        }
        loop {
            {
                table = cvar.wait(table).unwrap();
                match table.remove(&counter_pop) {
                    Some(v) => return v,
                    None => (),
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::Bencher;
    use std::{thread, time};

    #[test]
    fn test_jobqueue_1() {
        let q = StableJobResultQueue::<u64, u64>::new();

        q.push_job(1);
        q.push_job(2);
        q.push_job(3);

        let q0: StableJobResultQueue<u64, u64> = q.clone();
        thread::spawn(move || {
            // Pop first, insert last
            let j = q0.pop_jobhandler();
            thread::sleep(time::Duration::from_millis(100));
            j.commit(j.job * 10)
        });

        let q1 = q.clone();
        thread::spawn(move || {
            // Pop mid, insert mid
            thread::sleep(time::Duration::from_millis(25));
            let j = q1.pop_jobhandler();
            thread::sleep(time::Duration::from_millis(50));
            j.commit(j.job * 10)
        });

        let q2 = q.clone();
        thread::spawn(move || {
            // Pop last, insert first
            thread::sleep(time::Duration::from_millis(50));
            let j = q2.pop_jobhandler();
            j.commit(j.job * 10)
        });

        assert_eq!(q.pop_result(), 10);
        assert_eq!(q.pop_result(), 20);
        assert_eq!(q.pop_result(), 30);
    }

    #[bench]
    pub fn bench_jobqueue_100_batch(b: &mut Bencher) {
        let thread_count = 100;
        let batch_size = 50;
        let q = StableJobResultQueue::<Option<u32>, u32>::new();

        for _ in 0..thread_count {
            let q = q.clone();
            thread::spawn(move || {
                loop {
                    // Pop last, insert first
                    thread::sleep(time::Duration::from_millis(50));
                    let j = q.pop_jobhandler();
                    if j.job.is_none() {
                        return;
                    }
                    j.commit(j.job.unwrap() * 10);
                }
            });
        }

        b.iter(|| {
            for i in 0..batch_size {
                q.push_job(Some(i * 2));
            }
            for i in 0..batch_size {
                assert_eq!(q.pop_result(), i * 2 * 10);
            }
        });

        for _ in 0..thread_count {
            q.push_job(None);
        }
    }
}
