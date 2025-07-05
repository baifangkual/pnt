use std::fmt::Display;
use thiserror::Error;

/// tui运行过程的错误
#[derive(Debug, Error)]
pub enum TError {
    /// 主密码重试到最大次数仍未正确
    #[error("re try max exceed: {0}")]
    ReTryMaxExceed(u8),
}


/// 加密解密错误
#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("failed to encrypt main password")]
    EncryptMainPwd(Option<argon2::password_hash::Error>),
    #[error("illegal decode salt")]
    DecodeSalt(argon2::password_hash::Error),
    #[error("illegal decode main password")]
    DecodeMP(base64ct::Error)
}

/// 校验失败
#[derive(Debug, Error)]
pub struct VerifyError;
impl Display for VerifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "verify failed")
    }
}