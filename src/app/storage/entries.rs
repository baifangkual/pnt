use crate::app::entry::{EncryptedEntry, ValidEntry};
use crate::app::errors::AppError;
use crate::app::storage::{Storage, sql_result_map_to_option};
use chrono::{DateTime, Local};
use rusqlite::{Result as SqlResult, Row, params};

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

impl Storage {
    /// 模板-插入密码的 Sqlite 语句
    const INSERT_ENTRY_SQL: &'static str = r#"INSERT INTO "entry" ("about", "notes", "k", "v") VALUES (?, ?, ?, ?)"#;
    /// 模板-更新实体的 Sqlite 语句
    const UPDATE_ENTRY_SQL: &'static str =
        r#"UPDATE "entry" SET "about"=?, "notes"=?, "k"=?, "v"=?, "ut"=datetime('now', 'localtime') WHERE "id"=?"#;
    /// 模板-删除实体的 Sqlite 语句
    const DELETE_ENTRY_SQL: &'static str = r#"DELETE FROM "entry" WHERE "id"=?"#;

    /// 插入一条密码记录
    pub fn insert_entry(&self, insert_entry: &ValidEntry) {
        self.conn
            .execute(
                Self::INSERT_ENTRY_SQL,
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
    pub fn update_entry(&self, update_entry: &ValidEntry, id: u32) {
        self.conn
            .execute(
                Self::UPDATE_ENTRY_SQL,
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
    pub fn delete_entry(&self, entry_id: u32) {
        self.conn
            .execute(Self::DELETE_ENTRY_SQL, params![entry_id])
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
        let mut stmt = self.conn.prepare("SELECT * FROM entry WHERE about LIKE ?").unwrap();
        let rows = stmt.query_map([nl], row_map_entry).expect("Failed to select entry");
        rows.filter_map(sql_result_map_to_option).collect()
    }
    /// 查询所有entry
    pub fn select_all_entry(&self) -> Vec<EncryptedEntry> {
        let mut stmt = self.conn.prepare("SELECT * FROM entry").unwrap();
        let rows = stmt.query_map([], row_map_entry).expect("Failed to select entry");
        rows.filter_map(sql_result_map_to_option).collect()
    }

    /// 查询entry数量
    pub fn select_entry_count(&self) -> u32 {
        let r = self.conn.query_row("SELECT COUNT(*) FROM entry", [], |row| row.get(0));
        sql_result_map_to_option(r).unwrap_or_else(|| panic!("{}", AppError::DataCorrupted)) // 一定有值，因为表已初始化，若无则说明被破坏，直接panic
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::storage::Storage;
    #[test]
    fn test_db() {
        let db = Storage::open_in_memory().unwrap();
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
        let vec = db.select_all_entry();
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
