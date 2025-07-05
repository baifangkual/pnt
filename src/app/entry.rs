use crate::app::crypto::Decrypter;
use anyhow::Context;
use chrono::{DateTime, Local};

/// 完全映射用户的输入
/// 其中 identity and password 尚未加密
#[derive(Debug, Default, Clone)]
pub struct InputEntry {
    pub name: String,
    pub description: String,
    pub identity: String,
    pub password: String,
}
impl InputEntry {
    /// 验证当前状态是否合法，只有返回true才可进行加密及存储
    pub fn validate(&self) -> bool {
        // 名称和认证字段不能为空，不应判定trim后是否为空，因为这是刻意输入的
        !self.name.is_empty() && !self.identity.is_empty() && !self.password.is_empty()
    }
}


/// 一个待插入的条目，与数据库中一个条目相关
/// 一个用户输入的Entry若能够通过验证，则会转换为该类型
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ValidEntry {
    pub name: String,
    pub description: Option<String>,
    pub encrypted_identity: String,
    pub encrypted_password: String,
}

// /// 详情
// /// todo 若 DetailScreen 使用该类型则 生命周期声明会传染
// ///  妈的，后续或修改
// #[derive(Debug)]
// pub struct DetailEntry<'a> {
//     pub name: &'a str,
//     pub description: &'a str,
//     pub identity: &'a str,
//     pub password: &'a str,
// }

/// 一个条目，与数据库中一个条目相关
/// 条目中相关字段已加密
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct EncryptedEntry {
    /// 该条目的id readonly
    pub id: u32,
    /// 该条目的名称
    pub name: String,
    pub description: Option<String>,
    /// 认证字段 - k
    pub encrypted_identity: String,
    /// 密码字段 - v
    pub encrypted_password: String,
    /// 创建时间
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}
/// 实现排序，按照修改时间排序
impl EncryptedEntry {
    pub fn sort_by_update_time(left: &EncryptedEntry, right: &EncryptedEntry) -> std::cmp::Ordering {
        right.created_at.cmp(&left.created_at)
    }
    /// 解密 Entry 为 UserInputEntry
    pub fn decrypt<'a, Dec>(&'a self, decrypt: &Dec) -> anyhow::Result<InputEntry>
    where
        Dec: Decrypter<&'a EncryptedEntry, InputEntry>,
    {
        decrypt.decrypt(&self)
            .with_context(|| "decrypt entry failed")
    }
}
