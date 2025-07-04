use crate::app::entry::{Entry, UserInputEntry, ValidInsertEntry};
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use base64ct::{Base64, Encoding};
use std::convert::Infallible;

/// 加密器
pub trait Encrypter<P, C> {
    type EncrypterError: std::error::Error;
    /// 加密方法 - 将明文(plaintext)转为密文(ciphertext)
    fn encrypt(&self, plaintext: P) -> Result<C, Self::EncrypterError>;
}
/// 解密器
pub trait Decrypter<C, P> {
    type DecrypterError: std::error::Error;
    /// 解密方法 - 将密文(ciphertext)转为明文(plaintext)
    fn decrypt(&self, ciphertext: C) -> Result<P, Self::DecrypterError>;
}

pub struct NoEncrypter;
impl Encrypter<UserInputEntry, ValidInsertEntry> for NoEncrypter {
    type EncrypterError = Infallible;
    fn encrypt(&self, plaintext: UserInputEntry) -> Result<ValidInsertEntry, Self::EncrypterError> {
        Ok(ValidInsertEntry {
            name: plaintext.name,
            description: if plaintext.description.is_empty() {
                None
            } else {
                Some(plaintext.description)
            },
            encrypted_identity: plaintext.identity,
            encrypted_password: plaintext.password,
        })
    }
}
impl Decrypter<Entry, UserInputEntry> for NoEncrypter {
    type DecrypterError = Infallible;
    fn decrypt(&self, ciphertext: Entry) -> Result<UserInputEntry, Self::DecrypterError> {
        Ok(
            UserInputEntry {
                name: ciphertext.name,
                description: ciphertext.description.unwrap_or_default(),
                identity: ciphertext.encrypted_identity,
                password: ciphertext.encrypted_password
            }
        )
    }
}

/// 主密码加密器，使用Argon2id算法加密主密码明文
/// 返回的加密后为单向hash的b64编码
pub struct MainPwdEncrypter {
    salt: SaltString,
}
impl MainPwdEncrypter {
    pub fn from_salt(salt: &str) -> Self {
        Self {
            salt: SaltString::encode_b64(salt.as_bytes()).expect("encode salt failed"),
        }
    }
}
impl From<&str> for MainPwdEncrypter {
    fn from(salt: &str) -> Self {
        Self::from_salt(salt)
    }
}

impl Encrypter<String, String> for MainPwdEncrypter {
    type EncrypterError = Infallible;
    /// 加密主密码，使用Argon2id算法单向加密，后续仅校验hash，
    /// 该方法内应消耗主密码内存段覆写
    /// # Panics
    /// salt 太短 <8 或 太长 >64
    /// h_pass > usize::MAX/4
    fn encrypt(&self, plaintext: String) -> Result<String, Infallible> {
        let h_pass = Argon2::default()
            .hash_password(plaintext.as_bytes(), &self.salt)
            .expect("hash password failed")
            .to_string();
        Ok(Base64::encode_string(h_pass.as_bytes()))
    }
}

/// 主密码校验器
pub struct MainPwdVerifier {
    salt: SaltString,
    _pass_hash: String,
}
impl MainPwdVerifier {
    /// 构建一个主密码校验器
    /// # Arguments
    /// * `salt` - 盐
    /// * `cipher_pwd` - argon2 hash 加密后的主密码
    pub fn from_salt_and_passwd(salt: &str, pass_hash_b64: String) -> Self {
        let ub = Base64::decode_vec(&pass_hash_b64).expect("b64 decode failed");
        Self {
            salt: SaltString::encode_b64(salt.as_bytes()).expect("encode salt failed"),
            _pass_hash: String::from_utf8(ub).expect("utf8 decode failed"),
        }
    }
}

impl MainPwdVerifier {
    pub fn verify(&self, passwd: &str) -> bool {
        // argon2 实例仅是值容器，创建代价小，无需存储实例
        // 其param使用 pub const DEFAULT，编译时确定
        Argon2::default()
            .verify_password(
                passwd.as_bytes(),
                &PasswordHash::new(&self._pass_hash).expect("invalid hash"),
            )
            .is_ok()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_main_pwd() {
        let plaintext = "Hello, world!".to_owned();
        let encrypter = MainPwdEncrypter::from("hello world");
        let cs1 = encrypter.encrypt(plaintext.clone()).unwrap();
        let cs2 = encrypter.encrypt(plaintext.clone()).unwrap();
        assert_eq!(cs1, cs2);
        let cs3 = MainPwdEncrypter::from("hello world")
            .encrypt(plaintext)
            .unwrap();
        assert_eq!(cs3, cs1);
        // println!("cs1: {cs1}");
    }
    #[test]
    fn test_main_pwd_verify() {
        let plaintext = "pass".to_owned();
        let encrypter = MainPwdEncrypter::from("salt1111");
        let cs1 = encrypter.encrypt(plaintext.clone()).unwrap();
        let verifier = MainPwdVerifier::from_salt_and_passwd("salt1111", cs1);
        assert_eq!(verifier.verify(&plaintext), true);
        assert_eq!(verifier.verify("pas1"), false);
    }
}
