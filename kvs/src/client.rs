use std::{
    io::{BufReader, BufWriter, Write}, 
    net::{TcpStream, ToSocketAddrs}
};

use serde::Deserialize;
use serde_json::de::{Deserializer,IoRead};
use crate::{Result, common::{Request, GetResponse, SetResponse, RemoveResponse}, KvsError};

/// kvs 客户端
pub struct KvsClient {
    reader: Deserializer<IoRead<BufReader<TcpStream>>>,
    writer: BufWriter<TcpStream>
}

impl KvsClient {
    /// 连接服务端
    pub fn connect<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        let tcp_reader = TcpStream::connect(addr)?;
        let tcp_writer = tcp_reader.try_clone()?;
        Ok(KvsClient { 
            reader: Deserializer::from_reader(BufReader::new(tcp_reader)), 
            writer: BufWriter::new(tcp_writer) })
    }

    ///获取数据请求
    pub fn get(&mut self, key: String) -> Result<Option<String>>{
        serde_json::to_writer(&mut self.writer, &Request::Get { key })?;
        self.writer.flush()?;
        let resp = GetResponse::deserialize(&mut self.reader)?;
        match resp {
            GetResponse::Ok(value) => Ok(value),
            GetResponse::Err(e) => Err(KvsError::StringError(e))
        }

    }

    /// 添加数据请求
    pub fn set(&mut self, key: String, value: String) -> Result<()>{
        serde_json::to_writer(&mut self.writer, &Request::Set { key,value })?;
        self.writer.flush()?;
        let resp = SetResponse::deserialize(&mut self.reader)?;
        match resp {
            SetResponse::Ok(_) => Ok(()),
            SetResponse::Err(e) => Err(KvsError::StringError(e))
        }

    }

    /// 删除数据请求
    pub fn remove(&mut self, key: String) -> Result<()>{
        serde_json::to_writer(&mut self.writer, &&Request::Remove { key })?;
        self.writer.flush()?;
        let resp = RemoveResponse::deserialize(&mut self.reader)?;
        match resp {
            RemoveResponse::Ok(_) => Ok(()),
            RemoveResponse::Err(e) => Err(KvsError::StringError(e))
        }

    }
}