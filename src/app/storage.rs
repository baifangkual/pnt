use crate::app::consts::MAIN_PASS_KEY;
use crate::app::errors::AppError;
use crate::app::storage::sqlite::SqliteConn;

pub mod sqlite;

impl SqliteConn {
    /// 寻找盐-主密码
    pub fn query_b64_s_mph(&self) -> Option<String> {
        self.select_cfg_v_by_key(MAIN_PASS_KEY)
    }
    /// 存储给定的新值盐-主密码，存在则更新，不存在则插入
    pub fn store_b64_s_mph(&mut self, b64_salt_mph: &str) {
        if let Some(_) = self.query_b64_s_mph() {
            self.update_cfg(MAIN_PASS_KEY, b64_salt_mph)
        } else {
            self.insert_cfg(MAIN_PASS_KEY, b64_salt_mph)
        }
    }

    /// 检查是否未init mph
    ///
    /// 没有mph但有条目：这种情况说明数据文件被人为手动修改，非法情况，Err
    pub fn is_not_init_mph(&self) -> Result<bool, AppError> {
        if self.query_b64_s_mph().is_some() {
            return Ok(false); // 有，即已初始化，返回false
        };
        // 走到这里，没有主密码，判定是否已有条目，有条目即为非法状态
        if self.select_entry_count() == 0 {
            Ok(true)
        } else {
            Err(AppError::DataCorrupted)
        }
    }
}
