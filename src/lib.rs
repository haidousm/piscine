use std::sync::mpsc;

use std::sync::Arc;
use std::sync::Mutex;
pub struct ThreadPool {
    _handles: Vec<std::thread::JoinHandle<()>>,
    sender: mpsc::Sender<Box<dyn FnMut() + Send>>,
}

impl ThreadPool {
    pub fn new(num_threads: u8) -> Self {
        let (sender, receiver) = mpsc::channel::<Box<dyn FnMut() + Send>>();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut _handles = vec![];
        for _ in 0..num_threads {
            let clone = receiver.clone();
            let handle = std::thread::spawn(move || loop {
                let mut work = match clone.lock().unwrap().recv() {
                    Ok(work) => work,
                    Err(_) => break,
                };
                println!("Start");
                work();
                println!("Finish");
            });
            _handles.push(handle);
        }

        Self { _handles, sender }
    }
    pub fn execute<T: FnMut() + Send + 'static>(&self, work: T) {
        self.sender.send(Box::new(work)).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        use std::sync::atomic::{AtomicU32, Ordering};
        let n = AtomicU32::new(0);
        let nref = Arc::new(n);
        let pool = ThreadPool::new(8);
        let clone = nref.clone();
        let runner = move || {
            clone.fetch_add(1, Ordering::SeqCst);
        };
        pool.execute(runner.clone());
        pool.execute(runner);
        std::thread::sleep(std::time::Duration::from_secs(2));
        assert_eq!(nref.load(Ordering::SeqCst), 2);
    }
}
