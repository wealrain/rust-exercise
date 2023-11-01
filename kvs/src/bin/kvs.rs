use std::{process::exit, env::current_dir};

use clap::{Parser, Subcommand};
use kvs::{KvStore,Result};
use serde::de::value;

#[derive(Parser,Debug)]
#[command(author,version,about,long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>
}

#[derive(Subcommand,Debug)]
enum Commands {
    /// set the value of the key
    set {
        /// A string key
        KEY:String,
        /// A string value
        VALUE: String
    },
    /// get the value of a given key
    get {
        /// A string key
        KEY: String
    },
    /// remove a given key
    rm {
        /// A string key
        KEY: String
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::set { KEY, VALUE }) => {
            let mut store = KvStore::open(current_dir()?)?;
            store.set(KEY.to_string(), VALUE.to_string())?;
        }
        Some(Commands::get { KEY }) => {
            let mut store = KvStore::open(current_dir()?)?;
            if let Some(value) = store.get(KEY.to_string())? {
                println!("{}",value);
            } else {
                println!("Key not found");
            }
        }
        Some(Commands::rm { KEY }) => {
            let mut store = KvStore::open(current_dir()?)?;
            match store.remove(KEY.to_string()) {
                Ok(()) => {}
                Err(kvs::KvsError::KeyNotFound) => {
                    println!("Key not found");
                    exit(1);
                }
                Err(e) => return Err(e) 
            }
        }
        None=>{
            unreachable!();
        }
        
    }

    Ok(())
}