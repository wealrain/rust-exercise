use std::{
    collections::{HashMap, BTreeMap}, 
    path::{PathBuf, Path}, 
    fs::{File, self, OpenOptions}, 
    io::{Write, Seek, Read, BufWriter, BufReader, SeekFrom, self}, 
    ops::Range
};

use std::ffi::OsStr;


use serde::{Serialize, Deserialize};
use serde_json::Deserializer;

use crate::{KvsError,Result};

use super::KvsEngine;

const COMPACTION_THRESHOLD: u64 = 1024 * 1024;

/// KvStore存储String类型的kv键值对，使用BTreeMap进行存储
/// 支持set get rm操作
/// kv键值对将会被持久化存储在日志文件中
/// 采用BTreeMap 提高查询速度
pub struct KvStore {
    // 存储日志等数据的目录
    path: PathBuf,
    readers: HashMap<u64,BufReaderWithPos<File>>,
    writer: BufWriterWithPos<File>,
    // 当前日志名
    current_gen:u64,
    index:BTreeMap<String,CommandPos>,
    uncompacted: u64,
}

impl KvStore {
    /// 根据给定的路径打开一个kvStore
    /// 如果目录不存在则创建
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let path = path.into();
        fs::create_dir_all(&path)?;

        let mut readers = HashMap::new();
        let mut index = BTreeMap::new();

        // 加载并排序日志文件
        let gen_list = sorted_gen_list(&path)?;
        // 定义操作数据占用空间 (未压缩数据)
        let mut uncompacted = 0;

        // 遍历所有日志文件，将数据读取到内存中，并在内存中映射文件和文件流的关系
        for &gen in &gen_list {
            let mut reader = BufReaderWithPos::new(File::open(log_path(&path,gen))?)?;
            uncompacted += load(gen,&mut reader,&mut index)?;
            readers.insert(gen, reader);
        }

        // 创建新的日志文件进行数据读写
        let current_gen = gen_list.last().unwrap_or(&0) + 1;
        let writer = new_log_file(&path,current_gen,&mut readers)?;

        Ok(KvStore { path, readers, writer, current_gen, index, uncompacted })
    }

    /// 创建KvStore
    // pub fn new() -> KvStore {
    //     KvStore { map: HashMap::new() }
    // }

   

    /// 
    pub fn compact(&mut self) -> Result<()> {
        // 构建新的文件作为压缩后的日志存储，并将当前的操作执行新的日志文件
        let compaction_gen = self.current_gen + 1;
        self.current_gen += 2;
        self.writer = self.new_log_file(self.current_gen)?;
        let mut compaction_writer = self.new_log_file(compaction_gen)?;

        let mut new_pos = 0;
        // 遍历当前内存中所有的数据
        for cmd_pos in &mut self.index.values_mut() {
            let reader = self
                .readers
                .get_mut(&cmd_pos.gen)
                .expect("cannot find reader");
            if reader.pos != cmd_pos.pos {
                reader.seek(SeekFrom::Start(cmd_pos.pos))?;
            }

            let mut entry_reader = reader.take(cmd_pos.len);
            let len = io::copy(&mut entry_reader, &mut compaction_writer)?;
            *cmd_pos = (compaction_gen,new_pos..new_pos+len).into();
            new_pos += len;
        }

        compaction_writer.flush()?;

        let stale_gens: Vec<_> = self
            .readers
            .keys()
            .filter(|&&gen| gen < compaction_gen)
            .cloned()
            .collect();

        for stale_gen in stale_gens {
            self.readers.remove(&stale_gen);
            fs::remove_file(log_path(&self.path, stale_gen))?;
        }

        self.uncompacted = 0;

        Ok(())
    }

    fn new_log_file(&mut self,gen:u64) -> Result<BufWriterWithPos<File>> {
        new_log_file(&self.path, gen, &mut self.readers)
    }
}

impl KvsEngine for KvStore {
    /// 存储键值，如果键已存在，值将被覆盖
    fn set(&mut self,key:String,value:String) -> Result<()> {
        // self.map.insert(key, value);
        // 构建指令
        let cmd = Command::Set { key, value };
        // 将指令写入到日志文件的结尾
        let pos = self.writer.pos;
        serde_json::to_writer(&mut self.writer, &cmd)?;
        self.writer.flush()?;
        // 将数据存入到内存，并累加日志大小
        if let Command::Set { key, ..} = cmd {
            if let Some(old_cmd) = self.index.insert(key, (self.current_gen,pos..self.writer.pos).into()) {
                self.uncompacted += old_cmd.len;
            }
        }

        // 如果日志大小超过阈值进行压缩
        if self.uncompacted > COMPACTION_THRESHOLD {
            self.compact()?;
        }

        Ok(())

    }

    /// 从存储中获取值，值可能不存在
    fn get(&mut self,key:String) -> Result<Option<String>> {
        if let Some(cmd_pos) = self.index.get(&key) {
            let reader = self 
                .readers
                .get_mut(&cmd_pos.gen)
                .expect("Cannot find log reader");
            reader.seek(SeekFrom::Start(cmd_pos.pos))?;
            let cmd_reader = reader.take(cmd_pos.len);
            if let Command::Set {  value,.. } = serde_json::from_reader(cmd_reader)?{
                Ok(Some(value))
            } else {
                Err(KvsError::UnexceptedCommandType)
            }
        } else {
            Ok(None)
        }
    }

    /// 从存储中删除键值对
    fn remove(&mut self,key:String) -> Result<()> {
        if self.index.contains_key(&key) {
            let cmd = Command::Remove { key };
            serde_json::to_writer(&mut self.writer, &cmd)?;
            self.writer.flush()?;
            if let Command::Remove { key } = cmd {
                let old_cmd = self.index.remove(&key).expect("Key not found");
                self.uncompacted += old_cmd.len;
            }
            Ok(())
        } else {
            Err(KvsError::KeyNotFound)
        }
    }
}

/// 新建一个日志文件
fn new_log_file(path:&Path,gen:u64,readers:  &mut HashMap<u64, BufReaderWithPos<File>>) -> Result<BufWriterWithPos<File>> {
    let path = log_path(&path, gen);
    let writer = BufWriterWithPos::new(
        OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&path)?,
    )?;
    readers.insert(gen,BufReaderWithPos::new(File::open(&path)?)?);
    Ok(writer)
}

/// 加载日志文件，将日志数据转换为操作存储到内存中，并返回日志大小
fn load(gen: u64,reader: &mut BufReaderWithPos<File>,index: &mut BTreeMap<String,CommandPos>) -> Result<u64> {
    // 定位到文件头，按照Command进行反序列化读取
    let mut pos = reader.seek(SeekFrom::Start(0))?;
    let mut stream = Deserializer::from_reader(reader).into_iter::<Command>();
    let mut uncompacted = 0;

    while let Some(cmd) = stream.next() {
        let new_pos = stream.byte_offset() as u64;
        match cmd? {
            Command::Set { key, .. } => {
                if let Some(old_cmd) = index.insert(key, (gen,pos..new_pos).into()) {
                    uncompacted += old_cmd.len
                }
            }
            Command::Remove { key } => {
                if let Some(old_cmd) = index.remove(&key) {
                    uncompacted += old_cmd.len
                }
                uncompacted += new_pos - pos;
            }
        }
        pos = new_pos;
    }

    Ok(uncompacted)

}

fn log_path(dir:&Path,gen:u64) -> PathBuf {
    dir.join(format!("{}.log",gen))
}

// 对文件进行排序（按文件名）
fn sorted_gen_list(path: &Path) -> Result<Vec<u64>> {
    let mut gen_list:Vec<u64> = fs::read_dir(&path)?
        .flat_map(|res| -> Result<_> {Ok(res?.path())})
        .filter(|path| path.is_file() && path.extension() == Some("log".as_ref()))
        .flat_map(|path|{
            path.file_name()
                .and_then(OsStr::to_str)
                .map(|s| s.trim_end_matches(".log"))
                .map(str::parse::<u64>)  
        })
        .flatten()
        .collect(); 
    gen_list.sort_unstable();
    Ok(gen_list)
}

/// 存储的命令结构
#[derive(Serialize,Deserialize,Debug)]
enum Command {
  Set {key:String,value:String},
  Remove {key: String}
}

struct CommandPos {
    gen: u64,
    pos: u64,
    len: u64,
}

impl From<(u64,Range<u64>)> for CommandPos {
    fn from((gen,range): (u64,Range<u64>)) -> Self {
        CommandPos { 
            gen, 
            pos: range.start, 
            len: range.end - range.start 
        }
    }
}

struct  BufReaderWithPos<R: Read + Seek> {
    reader: BufReader<R>,
    pos: u64
}

impl<R: Read + Seek> BufReaderWithPos<R> {
    fn new(mut inner:R) -> Result<Self> {
        let pos = inner.seek(SeekFrom::Current(0))?;
        Ok(BufReaderWithPos { 
            reader: BufReader::new(inner), 
            pos: pos })
    }
}

impl<R: Read + Seek> Read for BufReaderWithPos<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = self.reader.read(buf)?;
        self.pos += len as u64;
        Ok(len)
    }
}

impl<R: Read + Seek> Seek for BufReaderWithPos<R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let pos = self.reader.seek(pos)?;
        self.pos = pos;
        Ok(self.pos)
    }
}

struct BufWriterWithPos<W: Write + Seek> {
    writer: BufWriter<W>,
    pos: u64
}

impl<W: Write + Seek> BufWriterWithPos<W> {
    fn new(mut inner: W) -> Result<Self> {
        let pos = inner.seek(SeekFrom::Current(0))?;
        Ok(BufWriterWithPos { 
            writer: BufWriter::new(inner), 
            pos: pos })
    }
}

impl<W: Write + Seek> Write for BufWriterWithPos<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let len = self.writer.write(buf)?;
        self.pos += len as u64;
        Ok(len)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl<W: Write + Seek> Seek for BufWriterWithPos<W> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let pos = self.writer.seek(pos)?;
        self.pos = pos;
        Ok(self.pos)
    }
}