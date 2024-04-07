use std::{thread, sync::{mpsc, Arc, Mutex}};

mod verbose;
type VerboseItem = verbose::Logger;


pub struct ThreadPool
{
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool
{
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool
    {
        assert!(size > 0);

        let (sender, reciever) = mpsc::channel();
        let reciever = Arc::new(Mutex::new(reciever));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size
        {
            workers.push(Worker::new(id, Arc::clone(&reciever)));
        }

        ThreadPool { workers, sender: Some(sender) }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool
{
    fn drop(&mut self)
    {
        drop(self.sender.take());

        for worker in &mut self.workers
        {
            VerboseItem::printmsg(VerboseItem::Worker, format!("Shutting down worker [{}]", worker.id));

            if let Some(thread) = worker.thread.take()
            {
                thread.join().unwrap();
            }
        }
    }
}


struct Worker
{
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker
{
    fn new(id: usize, reciever: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker
    {
        let thread = thread::spawn(move || loop
        {
            let message = reciever.lock().unwrap().recv();

            match message
            {
                Ok(job) =>
                {
                    VerboseItem::printmsg(VerboseItem::Worker, format!("Worker [{}] got a job, executing", id));

                    job();
                }
                Err(_) =>
                {
                    VerboseItem::printmsg(VerboseItem::Worker, format!("Worker [{}] disconnected, shutting down", id));
                    break;
                }
            }
        });

        Worker { id, thread: Some(thread) }
    }
}