use crate::app::errors::AppError;
use crate::app::storage::{Storage, sql_result_map_to_option};
use anyhow::Context;
use bitflags::bitflags;
use rusqlite::params;

bitflags! {
      pub struct BitCfg: u8 {
        /// 运行立即需要密码
        const VERIFY_ON_LAUNCH = 0b0000_0001;
        const IMMEDIATE_LOCK_SCREEN = 0b0000_0010;
        const _ = 0b1000_0000;
        // ... 预留其他
    }
}

impl Storage {
    /// 主密码存储名
    const KV_CFG_MAIN_PASS_KEY: &'static str = "mp";
    /// 寻找盐-主密码
    pub fn query_b64_s_mph(&self) -> Option<String> {
        self.select_cfg_v_by_key(Self::KV_CFG_MAIN_PASS_KEY)
    }

    /// 存储给定的新值盐-主密码，存在则更新，不存在则插入
    pub fn store_b64_s_mph(&self, b64_salt_mph: &str) {
        self.save_cfg(Self::KV_CFG_MAIN_PASS_KEY, b64_salt_mph)
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

    /// auto re-lock idle sec
    const AUTO_RE_LOCK_IDLE_SEC: &'static str = "ars";
    /// 若人为修改db文件导致 FromStr parse失败，则Err报告数据已损坏
    pub fn query_cfg_auto_relock_idle_sec(&self) -> anyhow::Result<Option<u32>> {
        self.select_cfg_v_by_key(Self::AUTO_RE_LOCK_IDLE_SEC)
            .map(|s| s.parse::<u32>())
            .transpose()
            .with_context(|| AppError::DataCorrupted)
    }
    /// 保存 auto_re_lock_idle_sec 配置
    /// 因为0（不auto）为默认值，遂走delete逻辑
    pub fn store_cfg_auto_re_lock_idle_sec(&self, auto_re_lock_idle_sec: u32) {
        if auto_re_lock_idle_sec == 0 {
            self.delete_cfg(Self::AUTO_RE_LOCK_IDLE_SEC);
        } else {
            self.save_cfg(Self::AUTO_RE_LOCK_IDLE_SEC, &auto_re_lock_idle_sec.to_string());
        }
    }

    /// auto close idle sec
    const AUTO_CLOSE_APP_IDLE_SEC: &'static str = "acs";
    /// 若人为修改db文件导致 FromStr parse失败，则Err报告数据已损坏
    pub fn query_cfg_auto_close_idle_sec(&self) -> anyhow::Result<Option<u32>> {
        self.select_cfg_v_by_key(Self::AUTO_CLOSE_APP_IDLE_SEC)
            .map(|s| s.parse::<u32>())
            .transpose()
            .with_context(|| AppError::DataCorrupted)
    }
    /// 保存 auto_close_idle_sec 配置
    /// 因为0（不auto）为默认值，遂走delete逻辑
    pub fn store_cfg_auto_close_idle_sec(&self, auto_close_idle_sec: u32) {
        if auto_close_idle_sec == 0 {
            self.delete_cfg(Self::AUTO_CLOSE_APP_IDLE_SEC)
        } else {
            self.save_cfg(Self::AUTO_CLOSE_APP_IDLE_SEC, &auto_close_idle_sec.to_string())
        }
    }

    /// bit flag key
    const BIT_FLAG_CFG_ID: &'static str = "bf";
    /// 查找 bit flag cfg值，如不存在，则返回Ok(None)
    ///
    /// 若人为修改db文件导致 FromStr parse失败，则Err报告数据已损坏
    pub fn query_cfg_bit_flags(&self) -> anyhow::Result<Option<BitCfg>> {
        let bf_or = self.select_cfg_v_by_key(Self::BIT_FLAG_CFG_ID);
        if bf_or.is_none() {
            return Ok(None);
        }
        bf_or
            .unwrap()
            .parse()
            .map(BitCfg::from_bits_truncate)
            .map(Some)
            .with_context(|| AppError::DataCorrupted) // 被人为修改db文件导致 parse失败时
    }

    /// 存储 bit flag 配置，覆盖或插入(依赖key值是否相同）
    pub fn store_cfg_bit_flags(&self, bf: BitCfg) {
        self.save_cfg(Self::BIT_FLAG_CFG_ID, &bf.bits().to_string())
    }

    // ===============================================================
    // SQL ===========================================================
    // ===============================================================

    /// 模板-查找内部配置 sql
    const SELECT_INNER_CFG_SQL: &'static str = r#"SELECT "v" FROM "cfg" WHERE "k"=?"#;

    /// 模板-插入内部配置 sql
    const SAVE_INNER_CFG_SQL: &'static str = r#"INSERT OR REPLACE INTO "cfg" ("k", "v") VALUES (?, ?)"#;

    /// 模板-删除内部配置 sql
    const DELETE_INNER_CFG_SQL: &'static str = r#"DELETE FROM "cfg" WHERE "k"=?"#;

    /// 插入配置 OR 更新配置 （取决于给定的key在table是否存在，存在则更新，不存在则插入）
    fn save_cfg(&self, key: &str, value: &str) {
        self.conn
            .execute(Self::SAVE_INNER_CFG_SQL, params![key, value])
            .expect("Failed to save cfg");
    }

    /// 删除配置
    fn delete_cfg(&self, key: &str) {
        self.conn
            .execute(Self::DELETE_INNER_CFG_SQL, params![key])
            .expect("Failed to delete cfg");
    }
    /// 通过key查询配置
    fn select_cfg_v_by_key(&self, key: &str) -> Option<String> {
        let r = self
            .conn
            .query_row(Self::SELECT_INNER_CFG_SQL, params![key], |row| row.get(0));
        sql_result_map_to_option(r)
    }
}
