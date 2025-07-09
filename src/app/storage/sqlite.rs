use crate::app::entry::{EncryptedEntry, ValidEntry};
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
    "about" TEXT NOT NULL,
    "notes" TEXT,
    "k" TEXT NOT NULL,
    "v" TEXT NOT NULL,
    "ct" TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
    "ut" TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
)"#;
// rusqlite 不会造成 sql 注入，因此无需使用参数化查询
/// 模板-插入密码的 Sqlite 语句
const INSERT_ENTRY_SQL: &str = r#"INSERT INTO "entry" ("about", "notes", "k", "v") VALUES (?, ?, ?, ?)"#;
/// 模板-更新实体的 Sqlite 语句
const UPDATE_ENTRY_SQL: &str =
    r#"UPDATE "entry" SET "about"=?, "notes"=?, "k"=?, "v"=?, "ut"=datetime('now', 'localtime') WHERE "id"=?"#;
/// 模板-删除实体的 Sqlite 语句
const DELETE_ENTRY_SQL: &str = r#"DELETE FROM "entry" WHERE "id"=?"#;

/// 存储密码的 Sqlite 数据库
pub struct SqliteConn {
    conn: Connection,
}

impl SqliteConn {
    /// 关闭连接，不再使用，该方法要求所有权
    pub fn close(self) {
        let _ = self.conn.close();
    }
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
fn row_map_entry(row: &Row) -> SqlResult<EncryptedEntry> {
    let id: u32 = row.get(0)?;
    let about: String = row.get(1)?;
    let notes: Option<String> = row.get(2)?;
    let encrypted_username: String = row.get(3)?;
    let encrypted_password: String = row.get(4)?;
    let created_time: DateTime<Local> = row.get(5)?;
    let updated_time: DateTime<Local> = row.get(6)?;
    Ok(EncryptedEntry {
        id,
        about,
        notes,
        encrypted_username,
        encrypted_password,
        created_time,
        updated_time,
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
        // 库文件加密 需要 bundled-sqlcipher，其需要openssl
        // conn.pragma_update(None, "key", "secret-keyXXXX")?;
        let mut s = Self { conn };
        s.init_tables_if_not_exists()?;
        Ok(s)
    }
    /// 使用内存
    #[cfg(test)]
    pub fn open_in_memory() -> SqlResult<Self> {
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
    pub fn insert_entry(&mut self, insert_entry: &ValidEntry) {
        self.conn
            .execute(
                INSERT_ENTRY_SQL,
                params![
                    insert_entry.about,
                    insert_entry.notes,
                    insert_entry.encrypted_username,
                    insert_entry.encrypted_password,
                ],
            )
            .expect("Failed to insert entry");
    }
    /// 更新一条密码记录
    pub fn update_entry(&mut self, update_entry: &ValidEntry, id: u32) {
        self.conn
            .execute(
                UPDATE_ENTRY_SQL,
                params![
                    update_entry.about,
                    update_entry.notes,
                    update_entry.encrypted_username,
                    update_entry.encrypted_password,
                    id // where
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
    pub fn select_entry_by_id(&self, id: u32) -> Option<EncryptedEntry> {
        let r = self
            .conn
            .query_one("SELECT * FROM entry WHERE id = ?", params![id], row_map_entry);
        sql_result_map_to_option(r)
    }
    /// 通过about模糊查询
    pub fn select_entry_by_about_like(&self, name_like: &str) -> Vec<EncryptedEntry> {
        let nl = format!("%{}%", name_like); // 左右
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM entry WHERE about LIKE ?")
            .expect("Failed to prepare query");
        let rows = stmt.query_map([nl], row_map_entry).expect("Failed to select entry");
        rows.filter_map(sql_result_map_to_option).collect()
    }
    /// 查询所有entry
    pub fn select_all_entry(&self) -> Vec<EncryptedEntry> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM entry")
            .expect("Failed to prepare query");
        let rows = stmt.query_map([], row_map_entry).expect("Failed to select entry");
        rows.filter_map(sql_result_map_to_option).collect()
    }

    /// 查询entry数量
    pub fn select_entry_count(&self) -> u32 {
        let r = self.conn.query_row("SELECT COUNT(*) FROM entry", [], |row| row.get(0));
        sql_result_map_to_option(r).unwrap() // 一定有值，因为表已初始化，若无则说明被破坏，直接panic
    }

    // =========== cfg ==============

    /// 插入配置
    pub(super) fn insert_cfg(&mut self, key: &str, value: &str) {
        self.conn
            .execute(INSERT_INNER_CFG_SQL, params![key, value])
            .expect("Failed to insert cfg");
    }

    /// 更新配置
    pub(super) fn update_cfg(&mut self, key: &str, value: &str) {
        self.conn
            .execute(UPDATE_INNER_CFG_SQL, params![value, key])
            .expect("Failed to update cfg");
    }

    /// 删除配置
    pub(super) fn delete_cfg(&mut self, key: &str) {
        self.conn
            .execute(DELETE_INNER_CFG_SQL, params![key])
            .expect("Failed to delete cfg");
    }
    /// 通过key查询配置
    pub(super) fn select_cfg_v_by_key(&self, key: &str) -> Option<String> {
        let r = self
            .conn
            .query_row("SELECT v FROM cfg WHERE k =?", params![key], |row| row.get(0));
        sql_result_map_to_option(r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_db() {
        let mut db = SqliteConn::open_in_memory().unwrap();
        let insert_e = ValidEntry {
            about: String::from("test"),
            notes: None,
            encrypted_username: String::from("test"),
            encrypted_password: String::from("test"),
        };

        // 精确到秒可能无意义
        let now: DateTime<Local> = DateTime::from(Local::now());
        // select
        db.insert_entry(&insert_e); // append ct, ut
        let mut vec = db.select_all_entry();
        assert_eq!(vec.len(), 1);
        let entry = vec[0].clone();
        assert_eq!(entry.id, vec[0].id);
        assert_eq!(entry.about, insert_e.about);
        assert_eq!(entry.notes, insert_e.notes);
        assert_eq!(entry.encrypted_username, insert_e.encrypted_username);
        assert_eq!(entry.encrypted_password, insert_e.encrypted_password);
        assert_eq!(entry.created_time, entry.updated_time);
        assert!(entry.created_time >= now);
        assert!(entry.updated_time >= now);
        // assert update
        let mut other_entry = entry.clone();
        other_entry.notes = Some(String::from("test"));
        let upd_entry = other_entry.clone();
        let v_e = ValidEntry {
            about: upd_entry.about,
            notes: upd_entry.notes,
            encrypted_username: upd_entry.encrypted_username,
            encrypted_password: upd_entry.encrypted_password,
        };
        db.update_entry(&v_e, other_entry.id);
        let after_update_query_by_id_one = db.select_entry_by_id(entry.id);
        assert!(after_update_query_by_id_one.is_some());
        let after_update = after_update_query_by_id_one.unwrap();
        assert_eq!(after_update.about, other_entry.about);
        assert_eq!(after_update.id, vec[0].id);
        assert_eq!(after_update.encrypted_username, other_entry.encrypted_username);
        assert_eq!(after_update.encrypted_password, other_entry.encrypted_password);
        assert_eq!(after_update.created_time, other_entry.created_time);
        assert!(after_update.updated_time >= now);
        assert_ne!(after_update.notes, entry.notes);
        // assert delete
        db.insert_entry(&insert_e);
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
