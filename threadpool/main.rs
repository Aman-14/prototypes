use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

type Job = Box<dyn FnOnce() + Send>;

pub struct ThreadPoolExecutor {
    sender: mpsc::Sender<Job>,
    workers: Vec<Worker>,
}

struct Worker {
    _id: i32,
    thread: Option<thread::JoinHandle<()>>,
}

impl ThreadPoolExecutor {
    pub fn new(num_workers: i32) -> Self {
        assert!(num_workers > 0);

        let (sender, receiver) = mpsc::channel::<Job>();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(num_workers as usize);
        for i in 0..num_workers {
            let receiver = receiver.clone();
            workers.push(Worker::new(i, receiver));
        }

        return Self { sender, workers };
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}

impl Drop for ThreadPoolExecutor {
    fn drop(&mut self) {
        for worker in &mut self.workers {
            if let Some(t) = worker.thread.take() {
                t.join().unwrap();
            }
        }
    }
}

impl Worker {
    fn new(id: i32, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
        let handle = thread::spawn(move || loop {
            let receiver = receiver.lock().unwrap();
            match receiver.try_recv() {
                Ok(job) => {
                    drop(receiver);
                    job();
                    println!("worker {} done", id);
                }
                Err(err) => {
                    drop(receiver);
                    // eprintln!("worker {} error: {:?}", id, err);
                    thread::sleep(std::time::Duration::from_millis(100));
                    continue;
                }
            }
        });

        return Self {
            _id: id,
            thread: Some(handle),
        };
    }
}

fn some_io() {
    println!("some io");
    thread::sleep(std::time::Duration::from_secs(10));
}

fn main() {
    let pool = ThreadPoolExecutor::new(4);
    pool.execute(|| some_io());
    pool.execute(|| some_io());
    pool.execute(|| some_io());
    pool.execute(|| some_io());
    pool.execute(|| some_io());
    pool.execute(|| some_io());
}
