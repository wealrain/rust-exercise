use std::process::exit;

use clap::{Parser, Subcommand};

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

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::set { KEY, VALUE }) => {
            eprintln!("unimplemented");
            exit(1);
        }
        Some(Commands::get { KEY }) => {
            eprintln!("unimplemented");
            exit(1);
        }
        Some(Commands::rm { KEY }) => {
            eprintln!("unimplemented");
            exit(1);
        }
        None=>{
            unreachable!();
        }
        
    }
}