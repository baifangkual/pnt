//! 使用 aes_gcm 库进行加密和解密

use crate::app::crypto::{Decrypter, Encrypter};
use crate::app::entry::{EncryptedEntry, InputEntry, ValidEntry};
use crate::app::error::{CryptoError};

struct StrEncrypter {
    
}
impl Encrypter<&str, String> for StrEncrypter {
    type EncrypterError = CryptoError;
    fn encrypt(&self, plaintext: &str) -> Result<String, Self::EncrypterError> {
        todo!()
    }
}
impl Decrypter<&str, String> for StrEncrypter {
    type DecrypterError = CryptoError;
    fn decrypt(&self, ciphertext: &str) -> Result<String, Self::DecrypterError> {
        todo!()
    }
}

pub struct EntryEncrypter {
    inner_enc : StrEncrypter, 
}

impl Encrypter<InputEntry, ValidEntry> for EntryEncrypter {
    type EncrypterError = CryptoError;
    fn encrypt(&self, input_entry: InputEntry) -> Result<ValidEntry, Self::EncrypterError> {
        todo!()
    }
}
impl Decrypter<EncryptedEntry, InputEntry> for EntryEncrypter {
    type DecrypterError = CryptoError;
    fn decrypt(&self, encrypted_entry: EncryptedEntry) -> Result<InputEntry, Self::DecrypterError> {
        todo!()
    }
}



#[cfg(test)]
mod test {
    use aes_gcm::{aead::{Aead, AeadCore, KeyInit, OsRng}, Aes256Gcm, Key};
    use std::str;
    use anyhow::{anyhow, Context};
    use argon2::PasswordHasher;
    use base64ct::{Base64, Encoding};

    #[test]
    fn test_aes_gcm() -> anyhow::Result<()> {
        // 示例明文
        let plaintext = "这是需要加密的绝密数据!";
        let my_key = "my_key";
        let salt = "my_salt----".as_bytes();
        let mut p_hash = [0u8; 32];
        argon2::Argon2::default()
            .hash_password_into(my_key.as_bytes(), salt, &mut p_hash)
            .map_err(|e| anyhow!(e))?;
        // ---------------------- 加密部分 ----------------------
        // 1. 生成随机密钥 (256位 = 32字节)
        // let key = Aes256Gcm::generate_key(&mut OsRng);
        let key = Key::<Aes256Gcm>::from_slice(&p_hash);
        // 2. 初始化加密器
        let cipher = Aes256Gcm::new(&key);

        // 3. 生成随机 nonce (12字节 - GCM标准长度)
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        // 4. 加密数据（自动附加认证标签）
        let ciphertext = cipher
            .encrypt(&nonce, plaintext.as_ref())
            .expect("加密失败");

        println!("加密成功！");
        println!("密钥 (Base64): {}", Base64::encode_string(&key));
        println!("Nonce (Base64): {}", Base64::encode_string(&nonce));
        println!("密文 (Base64): {}", Base64::encode_string(&ciphertext));

        // ---------------------- 解密部分 ----------------------
        // 1. 重新创建加密器（实际应用中需从安全存储获取密钥）
        let decipher = Aes256Gcm::new(&key);

        // 2. 解密数据（同时验证认证标签）
        let decrypted_data = decipher
            .decrypt(&nonce, ciphertext.as_ref())
            .expect("解密失败 - 认证标签无效或数据篡改！");

        // 3. 将解密后的字节转为字符串
        let decrypted_text = str::from_utf8(&decrypted_data)?;

        println!("\n解密成功！");
        println!("原始明文: {}", plaintext);
        println!("解密结果: {}", decrypted_text);

        Ok(())
    }
}