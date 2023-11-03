//! This module provides various thread pools. All thread pools should implement
//! the `ThreadPool` trait.
use crate::Result;

mod naive;
mod shared_queue;
mod rayon;


pub use naive::NaiveThreadPool;
pub use shared_queue::SharedQueueThreadPool;
pub use rayon::RayonThreadPool;

/// 定义线程池
pub trait ThreadPool {
    ///
    fn new(threads: u32) -> Result<Self> where Self: Sized;
    
    /// spawn a thread
    fn spawn<F>(&self,job:F) where F: FnOnce() + Send + 'static; 
}