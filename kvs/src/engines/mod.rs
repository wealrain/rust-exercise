use crate::Result;

/// kvs engine 定义
pub trait KvsEngine {
  /// 插入数据
  fn set(&mut self, key: String, value: String) -> Result<()>;
  /// 获取数据
  fn get(&mut self, key: String) -> Result<Option<String>>;
  /// 删除数据
  fn remove(&mut self, key: String) -> Result<()>;
}

mod kvs;
mod sled;

pub use self::kvs::KvStore;
pub use self::sled::SledKvsEngine;
