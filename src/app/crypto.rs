//! pnt 使用的加密解密部分

pub mod aes_gcm;

use anyhow::Context;
use crate::app::context::SecurityContext;
use crate::app::crypto::aes_gcm::EntryAes256GcmSecretEncrypter;
use crate::app::errors::{AppError, CryptoError};
use argon2::password_hash::{Error, SaltString};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::rand_core::{OsRng, RngCore};
use base64ct::{Base64, Encoding};

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

/// 主密码加密器，使用Argon2id算法加密主密码明文
/// 返回的加密后为单向hash的b64编码
pub struct MainPwdEncrypter {
    salt: SaltString,
}
impl MainPwdEncrypter {
    pub fn from_salt(salt: &[u8]) -> anyhow::Result<Self> {
        let enc = Self {
            salt: SaltString::encode_b64(salt).map_err(|e| CryptoError::DecodeSalt(e))?,
        };
        Ok(enc)
    }
    pub fn new_from_random_salt() -> anyhow::Result<Self> {
        let mut salt = [0u8; 32];
        OsRng.fill_bytes(&mut salt);
        Self::from_salt(&salt)
    }

    pub fn salt(&self) -> String {
        self.salt.to_string()
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
}
impl MainPwdVerifier {
    /// 构建一个主密码校验器
    /// # Arguments
    /// * `salt` - 盐
    /// * `b64_mph` - argon2 hash 加密后的主密码(b64编码）
    pub fn from_b64_s_mph(b64_s_mph: &String) -> anyhow::Result<Self> {
        // 从 s_mp_b64 可base64de到salt，若过程失败，则证明数据已被破坏
        let (salt, b64_mph) = decode_b64_salt_mph(&b64_s_mph)?;
        let ub = Base64::decode_vec(&b64_mph).map_err(|e| CryptoError::DecodeMP(e))?;
        Ok(Self {
            salt: SaltString::encode_b64(salt.as_bytes()).map_err(|e| CryptoError::DecodeSalt(e))?,
            mph: String::from_utf8(ub)?,
        })
    }
}

impl MainPwdVerifier {
    /// 校验主密码，返回 Result
    /// 若校验通过，则返回 Ok(true)
    /// 若校验失败，则返回 Ok(false)
    /// 若校验过程中出现错误，则返回 Err
    pub fn verify(&self, passwd: &str) -> anyhow::Result<bool> {
        // argon2 实例仅是值容器，创建代价小，无需存储实例
        // 其param使用 pub const DEFAULT，编译时确定
        let argon2 = Argon2::default();
        let verify_r = argon2.verify_password(
            passwd.as_bytes(),
            &PasswordHash::new(&self.mph).map_err(|e| CryptoError::EncryptMainPwd(e))?,
        );
        match verify_r {
            Ok(()) => Ok(true),
            Err(e) => {
                if let Error::Password = e {
                    Ok(false)
                } else {
                    Err(CryptoError::EncryptMainPwd(e))?
                }
            }
        }
    }
    /// 生成加密解密条目的密钥
    fn gen_key(&self, passwd: &str) -> anyhow::Result<[u8; 32]> {
        let mut gp = [0u8; 32];
        Argon2::default()
            .hash_password_into(passwd.as_bytes(), self.salt.as_str().as_bytes(), &mut gp)
            .map_err(|_| CryptoError::GenerateKey)?;
        Ok(gp)
    }

    /// 在校验成功后加载安全上下文，返回安全上下文
    /// 该方法不会对给定的密码再进行主密码校验
    pub fn load_security_context(&self, passwd: &str) -> anyhow::Result<SecurityContext> {
        Ok(SecurityContext::new(EntryAes256GcmSecretEncrypter::from_key(
            self.gen_key(passwd)?,
        )?))
    }
}

/// 将 B64_SALT:B64_MPH 解码为 (SALT, B64_MPH) 返回
///
/// 若数据被破坏则返回Err
pub fn decode_b64_salt_mph(b64_salt_mph: &str) -> anyhow::Result<(String, String)> {
    let mut sp = b64_salt_mph.split(':');
    let salt_b64 = sp.next()
        .ok_or(AppError::DataCorrupted)?;
    let salt = String::from_utf8(Base64::decode_vec(salt_b64)?)?;
    let mph_b64 = sp.next()
        .ok_or(AppError::DataCorrupted)?;
    Ok((salt, mph_b64.into()))
}
/// 将 SALT, B64_MPH 编码为 B64_SALT:B64_MPH 返回
pub fn encode_salt_b64_mph(salt: &str, b64_mph: &str) -> String {
    let b64_salt = Base64::encode_string(salt.as_bytes());
    format!("{}:{}", b64_salt, b64_mph)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_main_pwd() {
        let plaintext = "Hello, world!".to_owned();
        let salt = String::from("salt-string");
        let encrypter = MainPwdEncrypter::from_salt(salt.as_bytes()).unwrap();
        let cs1 = encrypter.encrypt(plaintext.clone()).unwrap();
        let cs2 = encrypter.encrypt(plaintext.clone()).unwrap();
        assert_eq!(cs1, cs2);
        let cs3 = MainPwdEncrypter::from_salt(salt.as_bytes())
            .unwrap()
            .encrypt(plaintext)
            .unwrap();
        assert_eq!(cs3, cs1);
        // println!("cs1: {cs1}");
    }
    // #[test]
    // fn test_main_pwd_verify() {
    //     let plaintext = "pass".to_owned();
    //     let salt = String::from("salt-string");
    //     let encrypter = MainPwdEncrypter::from_salt(salt.as_bytes()).unwrap();
    //     let cs1 = encrypter.encrypt(plaintext.clone()).unwrap();
    //     let verifier = MainPwdVerifier::from_salt_and_b64_mph(&encrypter.salt(), &cs1).unwrap();
    //     assert_eq!(verifier.verify(&plaintext).unwrap(), true);
    //     assert_eq!(verifier.verify("pas1").unwrap(), false);
    // }

    #[test]
    fn test_encode_salt_and_b64_mph(){
        let foobar = String::from("foobar");
        let encrypter = MainPwdEncrypter::new_from_random_salt().unwrap();
        let b64_mph = encrypter.encrypt(foobar.clone()).unwrap();
        let salt = encrypter.salt();
        let b64_s_mph = encode_salt_b64_mph(&salt, &b64_mph);
        let (salt_de, b64_mph_de) = decode_b64_salt_mph(&b64_s_mph).unwrap();
        assert_eq!(salt, salt_de);
        assert_eq!(b64_mph, b64_mph_de);
    }
}
