use std::fmt::Display;
use thiserror::Error;
use crate::{app::consts::MAIN_PASS_MAX_RE_TRY};

/// tui运行过程的错误
#[derive(Debug, Error)]
pub enum AppError {
    /// 主密码重试到最大次数仍未正确
    #[error("re-try max exceed: {max}", max = MAIN_PASS_MAX_RE_TRY)]
    ReTryMaxExceed,
    /// 找不到主密码
    #[error("main password not found")]
    MainPwdNotFound,
}


/// 加密解密错误
#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("failed to encrypt main password: {0}")]
    EncryptMainPwd(argon2::password_hash::Error),
    #[error("failed to decode salt: {0}")]
    DecodeSalt(argon2::password_hash::Error),
    #[error("failed to decode main password: {0}")]
    DecodeMP(base64ct::Error),
    #[error("generate key error")]
    GenerateKey,
    #[error("encrypt error: {0}")]
    Encrypt(aes_gcm::aead::Error),
    #[error("decrypt error: {0}")]
    Decrypt(aes_gcm::aead::Error),
    #[error("split ciphertext error")]
    CiphertextSplit,
    #[error("decode nonce error")]
    DecodeNonce,
    #[error("decode ciphertext error")]
    DecodeCiphertext,

}

/// 校验失败
#[derive(Debug, Error)]
pub struct VerifyError;
impl Display for VerifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "verify failed")
    }
}