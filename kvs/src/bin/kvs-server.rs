#[macro_use]
extern crate log;

use kvs::{*, thread_pool::*};
use std::{
    net::SocketAddr, 
    env::current_dir, 
    fs, 
    process::exit
};

use structopt::{StructOpt, clap::arg_enum};

const DEFAULT_LISTENING_ADDRESS: &str = "127.0.0.1:4000";
const DEFAULT_ENGINE: Engine = Engine::kvs;

arg_enum! {
    #[allow(non_camel_case_types)]
    #[derive(Debug,Copy,Clone,PartialEq,Eq)]
    enum Engine {
        kvs,
        sled
    }
}

#[derive(StructOpt,Debug)]
#[structopt(name = "kvs-server")]
struct Opt {
    #[structopt(
        long,
        help = "Sets the listening address",
        value_name = "IP:PORT",
        raw(default_value = "DEFAULT_LISTENING_ADDRESS"),
        parse(try_from_str)
    )]
    addr: SocketAddr,
    #[structopt(
        long,
        help = "Sets the storage engine",
        value_name = "ENGINE-NAME",
        raw(possible_values = "&Engine::variants()") 
    )]
    engine: Option<Engine>
}

fn main() {
    env_logger::builder().filter_level(log::LevelFilter::Info).init();

    let mut opt = Opt::from_args();
    let res = current_engine().and_then(move |curr_engine|{
        if opt.engine.is_none() {
            opt.engine = curr_engine;
        }
        if curr_engine.is_some() && opt.engine != curr_engine {
            error!("Wrong engine!");
            exit(1);
        }
        run(opt)
    });
    if let Err(e) = res {
        error!("{}",e);
        exit(1);
    }
}

fn run(opt:Opt) -> Result<()> {
    let engine = opt.engine.unwrap_or(DEFAULT_ENGINE);
    info!("kvs-server {}",env!("CARGO_PKG_VERSION"));
    info!("Storage engine: {}",engine);
    info!("Listening on {}",opt.addr);

    fs::write(current_dir()?.join("engine"), format!("{}",engine))?;
    let pool = RayonThreadPool::new(num_cpus::get() as u32)?;
    match engine {
        Engine::kvs => run_with(KvStore::open(current_dir()?)?,pool,opt.addr),
        Engine::sled => run_with(SledKvsEngine::new(
            sled::open(current_dir()?)?),pool, opt.addr)
    }
}

fn run_with<E:KvsEngine,P:ThreadPool>(engine:E,pool:P,addr:SocketAddr) -> Result<()> {
    
    let server = KvsServer::new(engine,pool);
    server.run(addr)
}

fn current_engine() -> Result<Option<Engine>> {
    let engine = current_dir()?.join("engine");
    if !engine.exists() {
        return Ok(None);
    }

    match fs::read_to_string(engine)?.parse() {
        Ok(engine) => Ok(Some(engine)),
        Err(e) => {
            warn!("The content of engine file is invalid: {}",e);
            Ok(None)
        }
    }
}