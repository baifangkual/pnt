//! pnt 使用的加密解密部分

pub mod aes_gcm;

use crate::app::entry::{EncryptedEntry, InputEntry, ValidEntry};
use crate::app::errors::CryptoError;
use anyhow::{anyhow, Context};
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use base64ct::{Base64, Encoding};
use std::convert::Infallible;

/// 加密器
pub trait Encrypter<P, C> {
    type EncrypterError: std::error::Error + Sync + Send + 'static;
    /// 加密方法 - 将明文(plaintext)转为密文(ciphertext)
    fn encrypt(&self, plaintext: P) -> Result<C, Self::EncrypterError>;
}
/// 解密器
pub trait Decrypter<C, P> {
    type DecrypterError: std::error::Error + Sync + Send + 'static;
    /// 解密方法 - 将密文(ciphertext)转为明文(plaintext)
    fn decrypt(&self, ciphertext: C) -> Result<P, Self::DecrypterError>;
}

pub struct NoEncrypter;
impl Encrypter<&InputEntry, ValidEntry> for NoEncrypter {
    type EncrypterError = Infallible;
    fn encrypt(&self, plaintext: &InputEntry) -> Result<ValidEntry, Self::EncrypterError> {
        Ok(ValidEntry {
            name: plaintext.name.clone(),
            description: if plaintext.description.is_empty() {
                None
            } else {
                Some(plaintext.description.clone())
            },
            encrypted_identity: plaintext.identity.clone(),
            encrypted_password: plaintext.password.clone(),
        })
    }
}
impl Decrypter<&EncryptedEntry, InputEntry> for NoEncrypter {
    type DecrypterError = Infallible;
    fn decrypt(&self, ciphertext: &EncryptedEntry) -> Result<InputEntry, Self::DecrypterError> {
        Ok(InputEntry {
            name: ciphertext.name.clone(),
            description: if let Some(desc) = &ciphertext.description {desc.clone()} else {String::new()},
            identity: ciphertext.encrypted_identity.clone(),
            password: ciphertext.encrypted_password.clone(),
        })
    }
}

/// 主密码加密器，使用Argon2id算法加密主密码明文
/// 返回的加密后为单向hash的b64编码
pub struct MainPwdEncrypter {
    salt: SaltString,
}
impl MainPwdEncrypter {
    pub fn from_salt(salt: &str) -> anyhow::Result<Self> {
        let enc = Self {
            salt: SaltString::encode_b64(salt.as_bytes())
                .map_err(|e| CryptoError::DecodeSalt(e))?,
        };
        Ok(enc)
    }
}

impl Encrypter<String, String> for MainPwdEncrypter {
    type EncrypterError = CryptoError;
    /// 加密主密码，使用Argon2id算法单向加密，后续仅校验hash，
    /// 该方法内应消耗主密码内存段覆写
    /// # Panics
    /// salt 太短 <8 或 太长 >64
    /// mph > usize::MAX/4
    fn encrypt(&self, plaintext: String) -> Result<String, CryptoError> {
        let mph = Argon2::default()
            .hash_password(plaintext.as_bytes(), &self.salt)
            .map_err(|e| CryptoError::EncryptMainPwd(e))?
            .to_string();
        Ok(Base64::encode_string(mph.as_bytes()))
    }
}

/// 主密码校验器
pub struct MainPwdVerifier {
    salt: SaltString,
    mph: String,
    gph: Option<[u8; 32]>,
}
impl MainPwdVerifier {
    /// 构建一个主密码校验器
    /// # Arguments
    /// * `salt` - 盐
    /// * `mph_b64` - argon2 hash 加密后的主密码
    pub fn from_salt_and_passwd_hash_b64(salt: &str, mph_b64: String) -> anyhow::Result<Self> {
        let ub = Base64::decode_vec(&mph_b64).map_err(|e| CryptoError::DecodeMP(e))?;
        Ok(Self {
            salt: SaltString::encode_b64(salt.as_bytes())
                .map_err(|e| CryptoError::DecodeSalt(e))?,
            mph: String::from_utf8(ub)?,
            gph: None,
        })
    }
}

impl MainPwdVerifier {
    pub fn verify(&mut self, passwd: &str) -> anyhow::Result<()> {
        // argon2 实例仅是值容器，创建代价小，无需存储实例
        // 其param使用 pub const DEFAULT，编译时确定
        let argon2 = Argon2::default();
        argon2
            .verify_password(
                passwd.as_bytes(),
                &PasswordHash::new(&self.mph).map_err(|e| CryptoError::EncryptMainPwd(e))?,
            )
            .map_err(|e| CryptoError::EncryptMainPwd(e))?;
        // gph gen
        let mut gp = [0u8; 32];
        argon2
            .hash_password_into(passwd.as_bytes(), self.salt.as_str().as_bytes(), &mut gp)
            .map_err(|_| CryptoError::GenerateKey)?;
        self.gph = Some(gp);
        Ok(())
    }

    pub fn mph(&self) -> &str {
        &self.mph
    }
    pub fn gph(&self) -> anyhow::Result<&[u8]> {
        match &self.gph {
            Some(gph) => Ok(gph),
            None => Err(anyhow!("not found, not verify main password")),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_main_pwd() {
        let plaintext = "Hello, world!".to_owned();
        let encrypter = MainPwdEncrypter::from_salt("hello world").unwrap();
        let cs1 = encrypter.encrypt(plaintext.clone()).unwrap();
        let cs2 = encrypter.encrypt(plaintext.clone()).unwrap();
        assert_eq!(cs1, cs2);
        let cs3 = MainPwdEncrypter::from_salt("hello world")
            .unwrap()
            .encrypt(plaintext)
            .unwrap();
        assert_eq!(cs3, cs1);
        // println!("cs1: {cs1}");
    }
    #[test]
    fn test_main_pwd_verify() {
        let plaintext = "pass".to_owned();
        let encrypter = MainPwdEncrypter::from_salt("salt1111").unwrap();
        let cs1 = encrypter.encrypt(plaintext.clone()).unwrap();
        let mut verifier = MainPwdVerifier::from_salt_and_passwd_hash_b64("salt1111", cs1).unwrap();
        assert!(verifier.verify(&plaintext).is_ok());
        assert!(verifier.verify("pas1").is_err());
    }

    #[test]
    fn test_slat_gen_mut_nonce() {
        let mp = "123456789101112".to_owned();
        let slat = "salt11111";
        let encrypter = MainPwdEncrypter::from_salt(slat).unwrap();
        let cs1 = encrypter.encrypt(mp.clone()).unwrap();
        let mut verifier = MainPwdVerifier::from_salt_and_passwd_hash_b64("salt1111", cs1).unwrap();
        let x = verifier.verify(&mp).is_ok();
        assert!(x);
        let x1 = verifier.gph().is_ok();
        assert!(x1);
    }
}
