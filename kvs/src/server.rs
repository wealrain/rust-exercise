use std::{
    net::{ToSocketAddrs, TcpListener, TcpStream}, 
    io::{BufReader, BufWriter,Write}
};

use serde_json::Deserializer;

use crate::{
    Result,
    KvsEngine, 
    common::{Request, GetResponse, SetResponse, RemoveResponse}
};

/// kvs 服务器端
pub struct KvsServer<E: KvsEngine> {
    engine: E
}

impl<E: KvsEngine> KvsServer<E> {
    /// 新建一个服务器
    pub fn new(engine: E) -> Self {
        KvsServer { engine }
    }

    /// 绑定IP地址，对外提供服务
    pub fn run<A: ToSocketAddrs>(mut self,addr: A) -> Result<()> {
        let listener = TcpListener::bind(addr)?;
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    if let Err(e) = self.serve(stream) {
                        error!("Error on serving client: {}",e)
                    }  
                }
                Err(e) => error!("Connection failed: {}",e)
            }
        }
        Ok(()) 
    }

    fn serve(&mut self, tcp: TcpStream) -> Result<()> {
        let peer_addr = tcp.peer_addr()?;
        let reader = BufReader::new(&tcp);
        let mut writer = BufWriter::new(&tcp);
        let req_reader = Deserializer::from_reader(reader).into_iter::<Request>();


        macro_rules! send_resp {
            ($resp:expr) => {{
                let resp = $resp;
                serde_json::to_writer(&mut writer,&resp)?;
                writer.flush()?;
                debug!("Response sent to {} : {:?}",peer_addr,resp);
            };};
        }

        for req in req_reader {
            let req = req?;
            debug!("Receive request from {}:{:?}",peer_addr,req);

            match req {
                Request::Get { key } => send_resp!(match self.engine.get(key) {
                    Ok(value) => GetResponse::Ok(value),
                    Err(e) => GetResponse::Err(format!("{}",e))
                }),
                Request::Set { key, value } => send_resp!(match self.engine.set(key,value) {
                    Ok(_) => SetResponse::Ok(()),
                    Err(e) => SetResponse::Err(format!("{}",e))
                }),
                Request::Remove { key } => send_resp!(match self.engine.remove(key) {
                    Ok(_) => RemoveResponse::Ok(()),
                    Err(e) => RemoveResponse::Err(format!("{}",e))
                })
                
            };
        }

        Ok(())

    }
}