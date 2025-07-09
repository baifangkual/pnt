use crate::app::consts::MAIN_PASS_KEY;
use crate::app::errors::AppError;
use crate::app::storage::sqlite::SqliteConn;

pub mod sqlite;



impl SqliteConn {
    /// 寻找盐-主密码，若找不到则Err
    pub fn query_b64_s_mph(&self) -> Result<String, AppError> {
        self.select_cfg_v_by_key(MAIN_PASS_KEY)
            .ok_or(AppError::DataCorrupted)
    }
    /// 存储给定的新值盐-主密码，存在则更新，不存在则插入
    pub fn store_b64_s_mph(&mut self, b64_salt_mph: &str) {
       if let Some(_) = self.select_cfg_v_by_key(MAIN_PASS_KEY) {
           self.update_cfg(MAIN_PASS_KEY, b64_salt_mph)
       } else {
           self.insert_cfg(MAIN_PASS_KEY, b64_salt_mph)
       }
        
    }
}