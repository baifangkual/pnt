use crate::app::cli::CliArgs;
use crate::app::config::Cfg;
use crate::app::consts::MAIN_PASS_KEY;
use crate::app::crypto::MainPwdVerifier;
use crate::app::crypto::aes_gcm::EntryAes256GcmSecretEncrypter;
use crate::app::errors::AppError;
use crate::app::storage::sqlite::SqliteConn;
use anyhow::{Context, anyhow};

/// 安全上下文，包含主密码校验器和条目加密解密器
pub struct SecurityContext {
    encrypter: EntryAes256GcmSecretEncrypter,
}
impl SecurityContext {
    pub fn new(encrypter: EntryAes256GcmSecretEncrypter) -> Self {
        Self { encrypter }
    }
}

/// 运行时程序上下文
pub struct PntContext {
    pub(crate) cfg: Cfg,
    pub(crate) cli_args: CliArgs,
    pub(crate) storage: SqliteConn,
    /// 只有输入了主密码的情况该字段才不为None
    pub(crate) security_context: Option<SecurityContext>,
}

impl PntContext {
    pub fn new_with_verified(
        cfg: Cfg, cli_args: CliArgs, storage: SqliteConn, security_context: SecurityContext,
    ) -> Self {
        Self {
            cfg,
            cli_args,
            storage,
            security_context: Some(security_context),
        }
    }
    pub fn new_with_un_verified(cfg: Cfg, cli_args: CliArgs, storage: SqliteConn) -> Self {
        Self {
            cfg,
            cli_args,
            storage,
            security_context: None,
        }
    }
    /// 读取cfg中salt和storage中主密码的哈希校验段，
    /// 构建 主密码校验器，
    /// 若主密码在storage中找不到或因salt等原因构建失败则返回Err
    pub fn build_mpv(&self) -> anyhow::Result<MainPwdVerifier> {
        let Some(mp_b64) = self.storage.select_cfg_v_by_key(MAIN_PASS_KEY) else {
            return Err(AppError::MainPwdNotFound.into());
        };
        Ok(MainPwdVerifier::from_salt_and_passwd_hash_b64(&self.cfg.salt, &mp_b64)?)
    }
    /// 检查是否已验证主密码
    pub fn is_verified(&self) -> bool {
        self.security_context.is_some()
    }
    /// 尝试获取条目加密解密器，若未验证主密码则返回Err
    pub fn try_encrypter(&self) -> anyhow::Result<&EntryAes256GcmSecretEncrypter> {
        match &self.security_context {
            Some(mpv) => Ok(&mpv.encrypter),
            None => Err(anyhow!("Main password is not verified")),
        }
    }
}

/// 笔记db状态，用于判断是否需要初始化
/// - NoStorage: db文件不存在，需要初始化
/// - NoMainPwd: db文件存在，但是主密码未设置，需要初始化
/// - Ready: db文件存在，主密码已设置，正常运行
pub enum NoteState {
    NoStorage,
    NoMainPwd,
    Ready,
}
impl NoteState {
    pub fn check(cfg: &Cfg) -> NoteState {
        // cfg 要求的位置不存在
        if !cfg.date.exists() {
            NoteState::NoStorage
        } else {
            // 存在，尝试读取主密码
            let conn = SqliteConn::new(&cfg.date)
                .with_context(|| format!("Failed to conn SQLite database: {}", cfg.date.display()))
                .unwrap();
            // 找不到主密码
            if conn.select_cfg_v_by_key(MAIN_PASS_KEY).is_none() {
                NoteState::NoMainPwd
            } else {
                // 存在主密码 但未校验
                NoteState::Ready
            }
        }
    }
}

/// 运行模式，Cli运行完立即退出 TUI直到明确退出信号
#[derive(Eq, PartialEq, Debug)]
pub enum RunMode {
    Cli,
    Tui,
}
