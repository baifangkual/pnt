use crate::app::cli::args::CliArgs;
use crate::app::config::Cfg;
use crate::app::encrypt::{MainPwdEncrypter, MainPwdVerifier};
use crate::app::entry::{Entry, UserInputEntry};
use crate::app::storage::sqlite::SqliteConn;
use anyhow::Context;

pub struct PntRuntimeContext {
    pub(crate) cfg: Cfg,
    pub(crate) cli_args: CliArgs,
    pub(crate) storage: SqliteConn,
    /// 主密码验证器，只有输入了主密码的情况该字段才不为None
    pub(crate) mpv: Option<MainPwdVerifier>,
    pub(crate) entries: Vec<Entry>,
}

impl PntRuntimeContext {
    pub fn new(
        cfg: Cfg,
        cli_args: CliArgs,
        storage: SqliteConn,
        mpv: Option<MainPwdVerifier>,
    ) -> Self {
        Self {
            cfg,
            cli_args,
            storage,
            mpv,
            entries: Vec::with_capacity(100),
        }
    }
}

pub const MAIN_PASS_KEY: &str = "mp";

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
