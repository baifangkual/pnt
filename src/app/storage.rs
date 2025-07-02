use std::convert::Infallible;

pub mod sqlite;



// 存储note中条目及 inner 配置
// 关联类型太多 
// pub trait Storage {
//     type EntryKey;
//     type ResidualEntry;
//     type FullEntry;
//     
//     fn store(&mut self, entry: Self::ResidualEntry);
//     fn update(&mut self, entry: Self::FullEntry);
//     fn delete(&mut self, key: Self::EntryKey);
//     fn load_by_key(&mut self, key: Self::EntryKey) -> Option<Self::FullEntry>;
//     fn load_all(&self) -> Vec<Self::FullEntry>;
//     
//     
// }