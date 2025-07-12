//! pnt 使用的加密解密部分

pub mod aes_gcm;

use crate::app::context::SecurityContext;
use crate::app::crypto::aes_gcm::EntryAes256GcmSecretEncrypter;
use crate::app::errors::{AppError, CryptoError};
use crate::app::storage::Storage;
use argon2::password_hash::rand_core::{OsRng, RngCore};
use argon2::password_hash::{Error, SaltString};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
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

/// 读取storage中salt和storage中主密码的哈希校验段，
/// 构建 主密码校验器，
/// 若主密码在storage中找不到或因salt等原因构建失败则返回Err
pub fn build_mpv(storage: &Storage) -> anyhow::Result<MainPwdVerifier> {
    let b64_s_mph = storage.query_b64_s_mph().ok_or(AppError::DataCorrupted)?;
    Ok(MainPwdVerifier::from_b64_s_mph(&b64_s_mph)?)
}

/// 主密码加密器，使用Argon2id算法加密主密码明文
/// 返回的加密后为单向hash的b64编码
pub struct MainPwdEncrypter {
    salt: [u8; 32],
}
impl MainPwdEncrypter {
    pub fn from_salt(salt: [u8; 32]) -> Self {
        Self { salt }
    }

    pub fn new_from_random_salt() -> Self {
        let mut salt = [0u8; 32];
        OsRng.fill_bytes(&mut salt);
        Self::from_salt(salt)
    }

    pub fn salt(&self) -> &[u8; 32] {
        &self.salt
    }
}

impl Encrypter<String, String> for MainPwdEncrypter {
    type EncrypterError = CryptoError;
    /// 加密主密码，使用Argon2id算法单向加密，后续仅校验hash，
    /// 返回 b64(salt32 + mph)
    /// # Panics
    /// salt 太短 <8 或 太长 >64
    /// mph > usize::MAX/4
    fn encrypt(&self, plaintext: String) -> Result<String, CryptoError> {
        let ss = SaltString::encode_b64(&self.salt).map_err(|e| CryptoError::DecodeSalt(e))?;
        let mph = Argon2::default()
            .hash_password(plaintext.as_bytes(), &ss)
            .map_err(|e| CryptoError::EncryptMainPwd(e))?
            .to_string();
        Ok(encode_b64_s_mph(&self.salt, &mph))
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
        let (salt, mph) = decode_b64_s_mph(&b64_s_mph)?;
        Ok(Self {
            salt: SaltString::encode_b64(&salt).map_err(|e| CryptoError::DecodeSalt(e))?,
            mph,
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
            .map_err(|_| CryptoError::GenerateKey)?; // 安全相关 用map_err 缩减 暴露的err信息
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

/// 将 b64(SALT(32) + MPH) 解码为 (SALT(32), MPH) 返回
///
/// 若数据被破坏则返回Err
fn decode_b64_s_mph(b64_s_mph: &str) -> anyhow::Result<([u8; 32], String)> {
    // 正常情况下不会 b64 decode 失败，只有当 文件被手动人为修改，才会有这种情况，遂向外告知数据已损坏
    let dec = Base64::decode_vec(b64_s_mph).map_err(|_| AppError::DataCorrupted)?;
    // 前32位为salt，后为utf8 mph
    let mut salt = [0_u8; 32];
    salt.copy_from_slice(&dec[..32]);
    let mph = &dec[32..];
    let mph = str::from_utf8(mph).map_err(|_| AppError::DataCorrupted)?.to_string();
    Ok((salt, mph))
}
/// 将 SALT(32), MPH 编码为 b64(SALT(32) + MPH) 返回
fn encode_b64_s_mph(salt: &[u8; 32], mph: &str) -> String {
    let mut vec = Vec::with_capacity(32 + mph.len());
    vec.extend(salt);
    vec.extend(mph.as_bytes());
    Base64::encode_string(&vec)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_encrypter_mph() {
        let plaintext = "Hello, world!".to_owned();
        let encrypter = MainPwdEncrypter::new_from_random_salt();
        let cs1 = encrypter.encrypt(plaintext.clone()).unwrap();
        let cs2 = encrypter.encrypt(plaintext.clone()).unwrap();
        let salt = encrypter.salt();
        assert_eq!(cs1, cs2);
        let cs3 = MainPwdEncrypter::from_salt(*salt).encrypt(plaintext).unwrap();
        assert_eq!(cs3, cs1);
        // println!("cs1: {cs1}");
    }

    #[test]
    fn test_encode_salt_and_b64_mph() {
        let foobar = String::from("foobar");
        let encrypter = MainPwdEncrypter::new_from_random_salt();
        let b64_mph = encrypter.encrypt(foobar.clone()).unwrap();
        let salt = encrypter.salt();
        let b64_s_mph = encode_b64_s_mph(&salt, &b64_mph);
        let (salt_de, mph) = decode_b64_s_mph(&b64_s_mph).unwrap();
        assert_eq!(*salt, salt_de);
        assert_eq!(b64_mph, mph);
    }

    #[test]
    fn test_gen_ne_key() {
        let foobar = String::from("foobar");
        let salt = SaltString::encode_b64(&[0; 32]).unwrap();
        let mut gp = [0u8; 32];
        Argon2::default()
            .hash_password_into(foobar.as_bytes(), salt.as_str().as_bytes(), &mut gp)
            .unwrap();
        let r_p = Argon2::default()
            .hash_password(foobar.as_bytes(), &salt)
            .unwrap()
            .to_string();
        let rpb = r_p.as_bytes();
        assert_ne!(gp, *rpb);
    }
}
