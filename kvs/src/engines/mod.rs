use crate::Result;

/// kvs engine 定义
pub trait KvsEngine: Clone + Send + 'static {
  /// 插入数据
  fn set(&self, key: String, value: String) -> Result<()>;
  /// 获取数据
  fn get(&self, key: String) -> Result<Option<String>>;
  /// 删除数据
  fn remove(&self, key: String) -> Result<()>;
}

mod kvs;
mod sled;

pub use self::kvs::KvStore;
pub use self::sled::SledKvsEngine;
