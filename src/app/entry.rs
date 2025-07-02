use crate::app::encrypt::{Decrypter, Encrypter};
use chrono::{DateTime, Local};

/// 完全映射用户的输入
/// 其中 identity and password 尚未加密
#[derive(Debug, Default)]
pub(crate) struct UserInputEntry {
    name: String,
    description: String,
    identity: String,
    password: String,
}
impl UserInputEntry {
    /// 验证当前状态是否合法，只有返回true才可进行加密及存储
    fn validate(&self) -> bool {
        // 名称和认证字段不能为空，不应判定trim后是否为空，因为这是刻意输入的
        !self.name.is_empty() && !self.identity.is_empty() && !self.password.is_empty()
    }
    /// 加密 UserInputEntry 为 ValidInsertEntry
    /// 当 UserInputEntry 不合法时，该方法会panic
    fn encrypt<Enc>(self, encrypt: &Enc) -> ValidInsertEntry
    where
        Enc: Encrypter<UserInputEntry, ValidInsertEntry>,
    {
        encrypt
            .encrypt(self)
            .unwrap_or_else(|e| panic!("UserInputEntry error: {e}"))
    }
}

/// 一个待插入的条目，与数据库中一个条目相关
/// 一个用户输入的Entry若能够通过验证，则会转换为ValidInsertEntry
#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct ValidInsertEntry {
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) encrypted_identity: String,
    pub(crate) encrypted_password: String,
}

/// 一个条目，与数据库中一个条目相关
#[derive(Debug, Eq, PartialEq, Clone)]
pub(crate) struct Entry {
    /// 该条目的id readonly
    pub(crate) id: u32,
    /// 该条目的名称
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    /// 认证字段 - k
    pub(crate) encrypted_identity: String,
    /// 密码字段 - v
    pub(crate) encrypted_password: String,
    /// 创建时间
    pub(crate) created_at: DateTime<Local>,
    pub(crate) updated_at: DateTime<Local>,
}
/// 实现排序，按照修改时间排序
impl Entry {
    fn sort_by_update_time(left: &Entry, right: &Entry) -> std::cmp::Ordering {
        left.created_at.cmp(&right.created_at)
    }
    /// 解密 Entry 为 UserInputEntry
    fn decrypt<Dec>(self, decrypt: &Dec) -> UserInputEntry
    where
        Dec: Decrypter<Entry, UserInputEntry>,
    {
        // Safety: 一个Entry 一定不会解密失败
        unsafe { decrypt.decrypt(self).unwrap_unchecked() }
    }
}
