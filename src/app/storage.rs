use crate::app::cfg::InnerCfg;
use crate::app::errors::AppError;
use crate::app::storage::kv_cfg::BitCfg;
use anyhow::anyhow;
use rusqlite::{Connection as sqliteConnection, Connection, Result as SqlResult};
use std::path::Path;

pub mod entries;
pub mod kv_cfg;

/// 内部配置表，以kv形式存储值，其中k为str类型主键
const CREATE_INNER_CFG_TABLE_SQL: &str = r#"CREATE TABLE IF NOT EXISTS "cfg" (
    "k" TEXT NOT NULL PRIMARY KEY,
    "v" TEXT
)"#;
/// 模板-检查库表 cfg entry 是否存在，应返回2
pub(super) const CHECK_TABLE_EXISTS: &str = r#"SELECT COUNT(*) FROM sqlite_master 
         WHERE type='table' 
         AND name IN ('cfg', 'entry')"#;
/// 模板-创建密码表的 Sqlite 语句
const CREATE_ENTRY_TABLE_TEMPLATE_SQL: &str = r#"CREATE TABLE IF NOT EXISTS "entry" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "about" TEXT NOT NULL,
    "notes" TEXT,
    "k" TEXT NOT NULL,
    "v" TEXT NOT NULL,
    "ct" TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
    "ut" TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
)"#;

/// 将 rusqlite::Result<T> 转换为 Option<T>，若查询返回无结果则返回None，若查询返回错误则panic
fn sql_result_map_to_option<T>(res: SqlResult<T>) -> Option<T> {
    match res {
        Ok(t) => Some(t),
        Err(rusqlite::Error::QueryReturnedNoRows) => None,
        Err(e) => panic!("{e:?}"),
    }
}

/// 存储数据的连接
pub struct Storage {
    conn: sqliteConnection,
}

impl Storage {
    /// 关闭连接，不再使用，该方法要求所有权
    pub fn close(self) {
        let _ = self.conn.close();
    }
}

impl Storage {
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

    /// 断言表 cfg entry 存在，都存在才Ok，该方法应在 [`Storage`] new 阶段调用
    fn assert_all_tables_exists(&self) -> anyhow::Result<()> {
        // 查询 SQLite 系统表以检查表是否存在
        let mut stmt = self.conn.prepare(CHECK_TABLE_EXISTS)?;
        let count: i32 = stmt.query_row([], |row| row.get(0))?;
        // 如果两个表都存在，计数应为 2
        if count != 2 {
            Err(AppError::DataCorrupted)? // 该方法
        } else {
            Ok(())
        }
    }

    /// 若表不存在则创建表
    pub(in crate::app::storage) fn init_tables_if_not_exists(&mut self) -> anyhow::Result<()> {
        self.conn.execute(CREATE_ENTRY_TABLE_TEMPLATE_SQL, [])?;
        self.conn.execute(CREATE_INNER_CFG_TABLE_SQL, [])?;
        Ok(())
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
