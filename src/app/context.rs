use crate::app::cfg::Cfg;
use crate::app::cli::CliArgs;
use crate::app::crypto::MainPwdVerifier;
use crate::app::crypto::aes_gcm::EntryAes256GcmSecretEncrypter;
use crate::app::errors::AppError;
use crate::app::storage::Storage;
use anyhow::Context;
use std::ops::Deref;
use std::path::Path;

/// 安全上下文，包含主密码校验器和条目加密解密器
pub struct SecurityContext {
    encrypter: EntryAes256GcmSecretEncrypter,
}
impl SecurityContext {
    pub fn new(encrypter: EntryAes256GcmSecretEncrypter) -> Self {
        Self { encrypter }
    }
}

impl Deref for SecurityContext {
    type Target = EntryAes256GcmSecretEncrypter;
    fn deref(&self) -> &Self::Target {
        &self.encrypter
    }
}

/// 运行时程序上下文
pub struct PntContext {
    pub(crate) cfg: Cfg,
    pub(crate) storage: Storage,
    /// 只有输入了主密码的情况该字段才不为None
    pub(crate) security_context: Option<SecurityContext>,
}

impl PntContext {
    pub fn new_with_verified(cfg: Cfg, storage: Storage, security_context: SecurityContext) -> Self {
        Self {
            cfg,
            storage,
            security_context: Some(security_context),
        }
    }
    pub fn new_with_un_verified(cfg: Cfg, storage: Storage) -> Self {
        Self {
            cfg,
            storage,
            security_context: None,
        }
    }
    pub fn is_need_mp_on_run(&self) -> bool {
        self.cfg.inner_cfg.need_main_passwd_on_run
    }

    /// 检查是否已验证主密码
    pub fn is_verified(&self) -> bool {
        self.security_context.is_some()
    }
    /// 尝试获取条目加密解密器，若未验证主密码则返回Err
    pub fn try_encrypter(&self) -> Result<&EntryAes256GcmSecretEncrypter, AppError> {
        match &self.security_context {
            Some(security_ctx) => Ok(security_ctx),
            None => Err(AppError::MainPwdNotVerified),
        }
    }
}

/// 笔记db状态，用于判断是否需要初始化
/// - NoStorage: db文件不存在，需要初始化
/// - NoMainPwd: db文件存在，但是主密码未设置，需要初始化
/// - Ready: db文件存在，主密码已设置，正常运行
pub enum DataFileState {
    NoStorage,
    NoMainPwd,
    Ready(Storage),
}
impl DataFileState {
    pub fn look(data_path: &Path) -> anyhow::Result<DataFileState> {
        // cfg 要求的位置不存在
        if !data_path.exists() {
            Ok(DataFileState::NoStorage)
        } else {
            // 存在，尝试读取主密码
            let conn = Storage::open_file(&data_path)?;
            // 找不到主密码
            if conn.is_not_init_mph()? {
                Ok(DataFileState::NoMainPwd)
            } else {
                // 存在主密码 但未校验
                Ok(DataFileState::Ready(conn))
            }
        }
    }
}
