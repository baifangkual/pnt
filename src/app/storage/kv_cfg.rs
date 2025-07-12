use crate::app::errors::AppError;
use crate::app::storage::{Storage, sql_result_map_to_option};
use anyhow::Context;
use bitflags::bitflags;
use rusqlite::params;

/// 模板-查找内部配置 sql
const SELECT_INNER_CFG_SQL: &str = r#"SELECT "v" FROM "cfg" WHERE "k"=?"#;

/// 模板-插入内部配置 sql
const SAVE_INNER_CFG_SQL: &str = r#"INSERT OR REPLACE INTO "cfg" ("key", "value") VALUES (?, ?)"#;

/// 模板-删除内部配置 sql
const DELETE_INNER_CFG_SQL: &str = r#"DELETE FROM "cfg" WHERE "k"=?"#;

// todo cfg 中 key v均应 hash，防止明确指向

/// 主密码存储名
pub const MAIN_PASS_KEY: &str = "mp";
/// bit flag key
const BIT_FLAG_CFG_ID: &str = "bf";

bitflags! {
      pub struct BitCfg: u8 {
        /// 运行立即需要密码
        const NEED_MAIN_ON_RUN = 0b0000_0001;
        const _ = 0b0000_0010;
        // ... 预留其他
    }
}

impl Storage {
    /// 寻找盐-主密码
    pub fn query_b64_s_mph(&self) -> Option<String> {
        self.select_cfg_v_by_key(MAIN_PASS_KEY)
    }

    /// 存储给定的新值盐-主密码，存在则更新，不存在则插入
    pub fn store_b64_s_mph(&mut self, b64_salt_mph: &str) {
        self.save_cfg(MAIN_PASS_KEY, b64_salt_mph)
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

    /// 查找 bit flag cfg值，如不能存在，则返回全0
    ///
    /// 若人为修改db文件导致 FromStr parse失败，则Err报告数据已损坏
    pub fn query_cfg_bit_flags(&self) -> anyhow::Result<BitCfg> {
        self.select_cfg_v_by_key(&BIT_FLAG_CFG_ID)
            .map(|bfv| bfv.parse())
            .unwrap_or_else(|| Ok(BitCfg::empty().bits())) // option unwrap // 未设定情况
            .map(BitCfg::from_bits_truncate)
            .with_context(|| AppError::DataCorrupted) // 被人为修改db文件导致 parse失败时
    }

    /// 存储 bit flag 配置，覆盖或插入(依赖key值是否相同）
    pub fn store_cfg_bit_flags(&mut self, bf: BitCfg) {
        self.save_cfg(&BIT_FLAG_CFG_ID, &bf.bits().to_string())
    }

    /// 插入配置 OR 更新配置 （取决于给定的key在table是否存在，存在则更新，不存在则插入）
    fn save_cfg(&mut self, key: &str, value: &str) {
        self.conn
            .execute(SAVE_INNER_CFG_SQL, params![key, value])
            .expect("Failed to save cfg");
    }

    /// 删除配置
    fn delete_cfg(&mut self, key: &str) {
        self.conn
            .execute(DELETE_INNER_CFG_SQL, params![key])
            .expect("Failed to delete cfg");
    }
    /// 通过key查询配置
    fn select_cfg_v_by_key(&self, key: &str) -> Option<String> {
        let r = self
            .conn
            .query_row(SELECT_INNER_CFG_SQL, params![key], |row| row.get(0));
        sql_result_map_to_option(r)
    }
}
