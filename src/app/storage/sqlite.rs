use crate::app::entry::{Entry, ValidInsertEntry};
use chrono::{DateTime, Local};
use rusqlite::{Connection, Result as SqlResult, Row, params};
use std::path::Path;

/// 内部配置表，以kv形式存储值，其中k为str类型主键
const CREATE_INNER_CFG_TABLE_SQL: &str = r#"CREATE TABLE IF NOT EXISTS "cfg" (
    "k" TEXT NOT NULL PRIMARY KEY,
    "v" TEXT
)"#;
/// 模板-插入内部配置的 Sqlite 语句
const INSERT_INNER_CFG_SQL: &str = r#"INSERT INTO "cfg" ("k", "v") VALUES (?,?)"#;
/// 模板-更新内部配置的 Sqlite 语句
const UPDATE_INNER_CFG_SQL: &str = r#"UPDATE "cfg" SET "v"=? WHERE "k"=?"#;
/// 模板-删除内部配置的 Sqlite 语句
const DELETE_INNER_CFG_SQL: &str = r#"DELETE FROM "cfg" WHERE "k"=?"#;

/// 模板-创建密码表的 Sqlite 语句
const CREATE_ENTRY_TABLE_TEMPLATE_SQL: &str = r#"CREATE TABLE IF NOT EXISTS "entry" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "name" TEXT NOT NULL,
    "desc" TEXT,
    "k" TEXT NOT NULL,
    "v" TEXT NOT NULL,
    "ct" TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),  -- 创建时间(ISO8601格式)
    "ut" TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))   -- 更新时间
)"#;
// rusqlite 不会造成 sql 注入，因此无需使用参数化查询
/// 模板-插入密码的 Sqlite 语句
const INSERT_ENTRY_SQL: &str =
    r#"INSERT INTO "entry" ("name", "desc", "k", "v") VALUES (?, ?, ?, ?)"#;
/// 模板-更新实体的 Sqlite 语句
const UPDATE_ENTRY_SQL: &str = r#"UPDATE "entry" SET "name"=?, "desc"=?, "k"=?, "v"=?, "ut"=datetime('now', 'localtime') WHERE "id"=?"#;
/// 模板-删除实体的 Sqlite 语句
const DELETE_ENTRY_SQL: &str = r#"DELETE FROM "entry" WHERE "id"=?"#;

/// 存储密码的 Sqlite 数据库
pub struct SqliteConn {
    conn: Connection,
}

/// 将 rusqlite::Result<T> 转换为 Option<T>，若查询返回无结果则返回None，若查询返回错误则panic
fn sql_result_map_to_option<T>(res: SqlResult<T>) -> Option<T> {
    match res {
        Ok(t) => Some(t),
        Err(rusqlite::Error::QueryReturnedNoRows) => None,
        Err(e) => panic!("{e:?}"),
    }
}
/// 将 Row 转换为 Entry
fn row_map_entry(row: &Row) -> SqlResult<Entry> {
    let id: u32 = row.get(0)?;
    let name: String = row.get(1)?;
    let description: Option<String> = row.get(2)?;
    let encrypted_identity: String = row.get(3)?;
    let encrypted_passwd: String = row.get(4)?;
    let created_at: DateTime<Local> = row.get(5)?;
    let updated_at: DateTime<Local> = row.get(6)?;
    Ok(Entry {
        id,
        name,
        description,
        encrypted_identity,
        encrypted_password: encrypted_passwd,
        created_at,
        updated_at,
    })
}
/// 将 Row 转换为 (k, v)
fn row_map_cfg_kv(row: &Row) -> SqlResult<(String, String)> {
    let key: String = row.get(0)?;
    let value: String = row.get(1)?;
    Ok((key, value))
}

impl SqliteConn {
    /// 指定数据库文件路径，建立连接
    pub fn new(path: &Path) -> SqlResult<Self> {
        let conn = Connection::open(path)?;
        let mut s = Self { conn };
        s.init_tables_if_not_exists()?;
        Ok(s)
    }
    /// 使用内存
    fn open_in_memory() -> SqlResult<Self> {
        let conn = Connection::open_in_memory()?;
        let mut s = Self { conn };
        s.init_tables_if_not_exists()?;
        Ok(s)
    }

    /// 若表不存在则创建表
    fn init_tables_if_not_exists(&mut self) -> SqlResult<()> {
        self.conn.execute(CREATE_ENTRY_TABLE_TEMPLATE_SQL, [])?;
        self.conn.execute(CREATE_INNER_CFG_TABLE_SQL, [])?;
        Ok(())
    }

    /// 插入一条密码记录
    pub fn insert_entry(&mut self, insert_entry: ValidInsertEntry) {
        self.conn
            .execute(
                INSERT_ENTRY_SQL,
                params![
                    insert_entry.name,
                    insert_entry.description,
                    insert_entry.encrypted_identity,
                    insert_entry.encrypted_password
                ],
            )
            .expect("Failed to insert entry");
    }
    /// 更新一条密码记录
    pub fn update_entry(&mut self, entry: &Entry) {
        self.conn
            .execute(
                UPDATE_ENTRY_SQL,
                params![
                    entry.name,
                    entry.description,
                    entry.encrypted_identity,
                    entry.encrypted_password,
                    entry.id // where
                ],
            )
            .expect("Failed to update entry");
    }

    /// 删除一条密码记录
    pub fn delete_entry(&mut self, entry_id: u32) {
        self.conn
            .execute(DELETE_ENTRY_SQL, params![entry_id])
            .expect("Failed to delete entry");
    }
    /// 通过id查询一条密码记录
    pub fn select_entry_by_id(&self, id: u32) -> Option<Entry> {
        let r = self.conn.query_one(
            "SELECT * FROM entry WHERE id = ?",
            params![id],
            row_map_entry,
        );
        sql_result_map_to_option(r)
    }
    /// 通过name模糊查询
    pub fn select_entry_by_name_like(&self, name_like: &str) -> Vec<Entry> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM entry WHERE name LIKE ?")
            .expect("Failed to prepare query");
        let rows = stmt
            .query_map(&[name_like], row_map_entry)
            .expect("Failed to select entry");
        rows.filter_map(|r| sql_result_map_to_option(r)).collect()
    }
    /// 查询所有entry
    pub fn select_all_entry(&self) -> Vec<Entry> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM entry")
            .expect("Failed to prepare query");
        let rows = stmt
            .query_map([], row_map_entry)
            .expect("Failed to select entry");
        rows.filter_map(|r| sql_result_map_to_option(r)).collect()
    }

    /// 插入配置
    pub fn insert_cfg(&mut self, key: &str, value: &str) {
        self.conn
            .execute(INSERT_INNER_CFG_SQL, params![key, value])
            .expect("Failed to insert cfg");
    }

    /// 更新配置
    pub fn update_cfg(&mut self, key: &str, value: &str) {
        self.conn
            .execute(UPDATE_INNER_CFG_SQL, params![value, key])
            .expect("Failed to update cfg");
    }

    /// 删除配置
    pub fn delete_cfg(&mut self, key: &str) {
        self.conn
            .execute(DELETE_INNER_CFG_SQL, params![key])
            .expect("Failed to delete cfg");
    }
    /// 通过key查询配置
    pub  fn select_cfg_v_by_key(&self, key: &str) -> Option<String> {
        let r = self
            .conn
            .query_row("SELECT v FROM cfg WHERE k =?", params![key], |row| {
                row.get(0)
            });
        sql_result_map_to_option(r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_db() {
        let mut db = SqliteConn::open_in_memory().unwrap();
        let insert_e = ValidInsertEntry {
            name: String::from("test"),
            description: None,
            encrypted_identity: String::from("test"),
            encrypted_password: String::from("test"),
        };

        // 精确到秒可能无意义
        let now: DateTime<Local> = DateTime::from(Local::now());
        // select
        db.insert_entry(insert_e.clone()); // append ct, ut
        let mut vec = db.select_all_entry();
        assert_eq!(vec.len(), 1);
        let entry = vec[0].clone();
        assert_eq!(entry.id, vec[0].id);
        assert_eq!(entry.name, insert_e.name);
        assert_eq!(entry.description, insert_e.description);
        assert_eq!(entry.encrypted_identity, insert_e.encrypted_identity);
        assert_eq!(entry.encrypted_password, insert_e.encrypted_password);
        assert_eq!(entry.created_at, entry.updated_at);
        assert!(entry.created_at >= now);
        assert!(entry.updated_at >= now);
        // assert update
        let mut other_entry = entry.clone();
        other_entry.description = Some(String::from("test"));
        db.update_entry(&other_entry);
        let after_update_query_by_id_one = db.select_entry_by_id(entry.id);
        assert!(after_update_query_by_id_one.is_some());
        let after_update = after_update_query_by_id_one.unwrap();
        assert_eq!(after_update.name, other_entry.name);
        assert_eq!(after_update.id, vec[0].id);
        assert_eq!(
            after_update.encrypted_identity,
            other_entry.encrypted_identity
        );
        assert_eq!(
            after_update.encrypted_password,
            other_entry.encrypted_password
        );
        assert_eq!(after_update.created_at, other_entry.created_at);
        assert!(after_update.updated_at >= now);
        assert_ne!(after_update.description, entry.description);
        // assert delete
        db.insert_entry(insert_e.clone());
        let vec2 = db.select_all_entry();
        assert_eq!(vec2.len(), 2);
        assert_ne!(vec2.get(0).unwrap().id, vec2.get(1).unwrap().id);

        let mut db_count = vec2.len();
        for x in &vec2 {
            db.delete_entry(x.id);
            let vec3 = db.select_all_entry();
            assert_eq!(vec3.len(), db_count - 1);
            db_count -= 1;
        }
    }
}
