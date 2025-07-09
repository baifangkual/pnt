use crate::app::crypto::Decrypter;
use anyhow::Context;
use chrono::{DateTime, Local};
use crate::app::errors::AppError;

/// 完全映射用户的输入
/// 其中 identity and password 尚未加密
#[derive(Debug, Default, Clone)]
pub struct InputEntry {
    pub about: String,
    pub notes: String,
    pub username: String,
    pub password: String,
}
impl InputEntry {
    /// 验证当前状态是否合法，只有返回true才可进行加密及存储
    pub fn validate(&self) -> bool {
        // 名称和认证字段不能为空，不应判定trim后是否为空，因为这是刻意输入的
        !self.about.is_empty() && !self.username.is_empty() && !self.password.is_empty()
    }
}

/// 一个待插入的条目，与数据库中一个条目相关
/// 一个用户输入的Entry若能够通过验证，则会转换为该类型
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ValidEntry {
    pub about: String,
    pub notes: Option<String>,
    pub encrypted_username: String,
    pub encrypted_password: String,
}

/// 一个条目，与数据库中一个条目相关
/// 条目中相关字段已加密
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct EncryptedEntry {
    /// 该条目的id readonly
    pub id: u32,
    /// 该条目的名称
    pub about: String,
    pub notes: Option<String>,
    /// 认证字段 - k
    pub encrypted_username: String,
    /// 密码字段 - v
    pub encrypted_password: String,
    /// 创建时间
    pub created_time: DateTime<Local>,
    pub updated_time: DateTime<Local>,
}
/// 实现排序，按照修改时间排序
impl EncryptedEntry {
    pub fn sort_by_update_time(left: &EncryptedEntry, right: &EncryptedEntry) -> std::cmp::Ordering {
        right.created_time.cmp(&left.created_time)
    }
    /// 解密 Entry 为 UserInputEntry
    pub fn decrypt<'a, Dec>(&'a self, decrypt: &Dec) -> anyhow::Result<InputEntry>
    where
        Dec: Decrypter<&'a EncryptedEntry, InputEntry>,
    {
        // 主要提示DataCorrupted：存储成功的我想象不会解密失败，唯一解释是实际文件被人为修改，即提示数据已损坏
        decrypt.decrypt(&self).with_context(|| AppError::DataCorrupted)
    }
}
