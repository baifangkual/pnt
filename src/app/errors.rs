use thiserror::Error;

/// tui运行过程的错误
#[derive(Debug, Error)]
pub enum AppError {
    /// 主密码重试到最大次数仍未正确
    #[error("invalid password")]
    InvalidPassword,
    /// 数据被破坏（即部分cf读取值失败，即说明手动修改了数据文件）
    ///
    /// 在使用pnt时，这是不允许的情况，程序应退出
    #[error("data is corrupted (the data file has been manually altered ?)")]
    DataCorrupted,
    /// 未校验主密码却到达了需要主密码的请求
    #[error("main password is not verified")]
    MainPwdNotVerified,
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
    #[error("invalid nonce length")]
    InvalidNonceLength,
}
