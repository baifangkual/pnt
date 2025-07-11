use crate::app::config::InnerCfg;
use crate::app::consts::MAIN_PASS_KEY;
use crate::app::errors::AppError;
use crate::app::storage::sqlite::SqliteConn;
use anyhow::anyhow;
use rusqlite::Connection;
use std::path::Path;

pub mod sqlite;

impl SqliteConn {
    /// 关闭连接，不再使用，该方法要求所有权
    pub fn close(self) {
        let _ = self.conn.close();
    }
    /// 将配置文件的 inner_cfg 填充
    pub(crate) fn fill_inner_cfg(&self, inner_cfg: &mut InnerCfg) -> anyhow::Result<()> {
        // todo 填充库中查询到的cfg到cfg中
        Ok(())
    }
}

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

    /// 将原有通过 open 在内存的数据库写入到磁盘指定位置的文件中,
    /// 若给定位置有存在的文件实体，则Err
    /// 该方法执行后，conn连接仍连接的内存数据库，遂该方法要求所有权消耗之
    pub fn db_mem_to_disk(self, disk_path: &Path) -> anyhow::Result<()> {
        if disk_path.exists() {
            Err(anyhow!("file '{}' already exists", disk_path.display().to_string()))
        } else {
            // 使用 VACUUM INTO 语句将内存数据库复制到磁盘
            let sql = format!("VACUUM INTO '{}'", disk_path.to_str().unwrap());
            self.conn.execute(&sql, [])?;
            Ok(())
        }
    }

    /// 指定数据库文件路径，建立连接, 该方法能Ok返回则表一定存在
    pub fn new(path: &Path) -> anyhow::Result<Self> {
        let conn = Connection::open(path)?;
        let s = Self { conn };
        s.assert_all_tables_exists()?;
        Ok(s)
    }

    /// 使用内存建立连接, 该方法能Ok返回则表一定存在
    pub fn open_in_memory() -> anyhow::Result<Self> {
        let conn = Connection::open_in_memory()?;
        let mut s = Self { conn };
        s.init_tables_if_not_exists()?;
        Ok(s)
    }

    /// 断言表 cfg entry 存在，都存在才Ok，该方法应在 [`SqliteConn`] new 阶段调用
    fn assert_all_tables_exists(&self) -> anyhow::Result<()> {
        // 查询 SQLite 系统表以检查表是否存在
        let mut stmt = self.conn.prepare(sqlite::CHECK_TABLE_EXISTS)?;
        let count: i32 = stmt.query_row([], |row| row.get(0))?;
        // 如果两个表都存在，计数应为 2
        if count != 2 {
            Err(AppError::DataCorrupted)? // 该方法
        } else {
            Ok(())
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     #[test]
//     fn test_backup_mem_to_disk() -> anyhow::Result<()> {
//         let conn = SqliteConn::open_in_memory()?;
//         conn.backup_memory_to_disk(Path::new("./target/mem_bak"))
//     }
// }
