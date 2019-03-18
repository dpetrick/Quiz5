//! Some things need implementing!

use crossbeam_deque::{self, Injector, Steal};
use std::{
  sync::Arc,
  thread::{spawn, JoinHandle},
};

pub struct ThreadPool {
  workers: Vec<Worker>,
  task_queue: Arc<Injector<Job>>,
}

impl ThreadPool {
  pub fn new(size: usize) -> Self {
    let task_queue = Arc::new(Injector::new());
    let workers: Vec<Worker> = (0..size)
      .map(|_| Worker::new(123, Arc::clone(&task_queue)))
      .collect();

    Self {
      task_queue,
      workers,
    }
  }

  pub fn shutdown(self) {
    for worker in self.workers {
      worker.shutdown();
    }
  }

  /// Queue some work to be run
  pub fn queue<F>(&mut self, f: F)
  where
    F: FnOnce() + Send + 'static,
  {
    let job = Box::new(f);
    self.task_queue.push(job);
  }
}

trait FnBox {
  fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
  fn call_box(self: Box<F>) {
    (*self)()
  }
}

/// An easy to use type that wraps around a static function
/// that is only called once, passed between threads.
///
/// This acts as the main Job type for our thread pool
type Job = Box<FnBox + Send + 'static>;

enum Signal {
  Terminate,
}

struct Worker {
  _thread: JoinHandle<()>,
  _signal_queue: crossbeam_deque::Worker<Signal>,
}

impl Worker {
  fn new(_id: usize, injector: Arc<Injector<Job>>) -> Self {
    let injector = Arc::clone(&injector);
    let signal_queue = crossbeam_deque::Worker::new_fifo();
    let signal_receiver = signal_queue.stealer();

    let handle = spawn(move || {
      // Check if thread should terminate
      loop {
        if let Steal::Success(Signal::Terminate) = signal_receiver.steal() {
          println!("Shutting down thread");
          break;
        };

        // Work off tasks
        if let Steal::Success(t) = injector.steal() {
          t.call_box()
        };
      }
    });

    Worker {
      _thread: handle,
      _signal_queue: signal_queue,
    }
  }

  pub fn shutdown(self) {
    self._signal_queue.push(Signal::Terminate);
    self._thread.join().unwrap();
  }
}
