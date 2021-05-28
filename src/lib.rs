use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

pub struct ThreadPool {
    // vector
    workers: Vec<Worker>,
    // channel
    sender: mpsc::Sender<Message>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job), // holds the Job the thread should run
    Terminate, // causes thread to exit its loop and stop
}

// TODO: Add more documentation to ThreadPool and its public methods
impl ThreadPool {
    /// Create a new ThreadPool
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    /// 
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        // put receiving end of channel in an Arc and a Mutex
        // for each new worker we clone the arc to bump the reference count so
        // the workers can share ownership of the receiving end
        let receiver = Arc::new(Mutex::new(receiver));

        // initialize threads vectory with capacity of size
        // with_capacity preallocates space in the vector
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        // store sending side of channel in the ThreadPool instance
        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");

        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }
        
        println!("Shutting down all workers.");

        // loop through each of the thread pool workers
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            // call lock on receiver to acquire the mutex, then unwrap to panic
            // on errors
            // if we get the lock on the mutex `recv` receives a Job from the 
            // channel
            // TODO: change this unwrap to an `expect` with error message
            let message = receiver.lock().unwrap().recv().unwrap();

            match message {
                Message::NewJob(job) => {
                    println!("Worker {} got a job; executing.", id);

                    job();
                }
                Message::Terminate => {
                    println!("Worker {} was told to terminate.", id);
                    
                    break;
                }
            }
        });

        Worker { id, thread: Some(thread) }
    }
}
