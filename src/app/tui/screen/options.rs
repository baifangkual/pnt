use anyhow::anyhow;
use crate::app::entry::EncryptedEntry;

/// 二分类枚举
#[derive(Debug, Clone)]
pub enum YN {
    Yes,
    No,
}

/// 带 YN 选项的实体，可载荷 Item
#[derive(Debug, Clone)]
pub struct OptionYN<C> {
    pub title: String,
    pub desc: String,
    pub content: Option<C>,
    pub yn: Option<YN>,
}

impl<C> OptionYN<C> {
    
    pub fn content(&self) -> Result<&C, anyhow::Error> {
        match self.content {
            Some(ref c) => Ok(c),
            None => Err(anyhow!("Content not found")),
        }
    }
    
    pub fn new_just_title_desc(title: &str, desc: &str) -> Self {
        OptionYN {
            title: title.into(),
            desc: desc.into(),
            content: None,
            yn: None,
        }
    }
    pub fn change_yn(&mut self, yn: YN) {
        self.yn = Some(yn);
    }
    pub fn set_content(&mut self, c: C) {
        self.content = Some(c);
    }
}

impl OptionYN<EncryptedEntry> {
    /// 删除页面用的
    pub fn new_delete_tip(encrypted_entry: EncryptedEntry) -> Self {
        let d_name = &encrypted_entry.name;
        let d_desc= encrypted_entry.description.as_ref().unwrap_or(&"_".to_string()).clone();
        OptionYN {
            title: format!("DELETE '{}' ?", d_name),
            desc: d_desc,
            content: Some(encrypted_entry),
            yn: None,
        }
    }
}
