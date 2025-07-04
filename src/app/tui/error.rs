
use thiserror::Error;

/// tui运行过程的错误
#[derive(Debug, Error)]
pub enum TError {
    /// 主密码重试到最大次数仍未正确
    #[error("re try max exceed: {0}")]
    ReTryMaxExceed(u8),
}