use std::collections::HashMap;

/// KvStore存储String类型的kv键值对，使用hashmap进行存储
/// 支持set get rm操作
/// 
#[derive(Default)]
pub struct KvStore {
    map: HashMap<String,String>,
}

impl KvStore {
    /// 创建KvStore
    pub fn new() -> KvStore {
        KvStore { map: HashMap::new() }
    }

    /// 存储键值，如果键已存在，值将被覆盖
    pub fn set(&mut self,key:String,value:String) {
        self.map.insert(key, value);
    }

    /// 从存储中获取值，值可能不存在
    pub fn get(&self,key:String) -> Option<String> {
        self.map.get(&key).cloned()
    }

    /// 从存储中删除键值对
    pub fn remove(&mut self,key:String) {
        self.map.remove(&key);
    }
}