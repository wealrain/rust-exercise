use std::thread;

use super::ThreadPool;
use crate::Result;

/// 并非真正意义上的线程池，每次都会产生一个新的线程
pub struct NaiveThreadPool;

impl ThreadPool for NaiveThreadPool {
    fn new(_threads: u32) -> Result<Self> {
        Ok(NaiveThreadPool)
    }

    fn spawn<F>(&self,job:F) where F: FnOnce() + Send + 'static {
        thread::spawn(job);
    }
}

