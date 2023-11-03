#![deny(missing_docs)]
//! A simple kv store

#[macro_use]
extern crate log;

pub use error::{KvsError,Result};
pub use engines::{KvStore,KvsEngine,SledKvsEngine};
pub use server::KvsServer;
pub use client::KvsClient;

mod error;
mod engines;
mod server;
mod common;
mod client;