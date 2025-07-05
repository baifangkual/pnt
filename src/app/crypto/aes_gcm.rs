//! 使用 aes_gcm 库进行加密和解密

use crate::app::crypto::{Decrypter, Encrypter, MainPwdVerifier};
use crate::app::entry::{EncryptedEntry, InputEntry, ValidEntry};
use crate::app::errors::CryptoError;
use aes_gcm::aead::consts::U12;
use aes_gcm::aead::generic_array::GenericArray;
use aes_gcm::aes::Aes256;
use aes_gcm::{aead::{Aead, AeadCore, KeyInit}, Aes256Gcm, AesGcm, Key, Nonce};
use anyhow::Result;
use base64ct::Encoding;

// struct StrAes256GcmEncrypter;
// impl Encrypter<&str, String> for StrAes256GcmEncrypter {
//     type EncrypterError = CryptoError;
//     fn encrypt(&self, plaintext: &str) -> Result<String, Self::EncrypterError> {
//         todo!()
//     }
// }
// impl Decrypter<&str, String> for StrAes256GcmEncrypter {
//     type DecrypterError = CryptoError;
//     fn decrypt(&self, ciphertext: &str) -> Result<String, Self::DecrypterError> {
//         todo!()
//     }
// }

pub struct EntryAes256GcmEncrypter {
    inner_enc : AesGcm<Aes256, U12>,
}
impl EntryAes256GcmEncrypter {
    pub fn new_from_main_pwd_verifier(main_pwd_verifier: &MainPwdVerifier) -> Result<EntryAes256GcmEncrypter> {
        let gk = main_pwd_verifier.gph()?;
        let gcm = aes_gcm::Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&gk));
        Ok(Self {inner_enc : gcm})
    }
}

impl Encrypter<&InputEntry, ValidEntry> for EntryAes256GcmEncrypter {
    type EncrypterError = CryptoError;
    fn encrypt(&self, input_entry: &InputEntry) -> Result<ValidEntry, Self::EncrypterError> {
        todo!()
    }
}
impl Decrypter<&EncryptedEntry, InputEntry> for EntryAes256GcmEncrypter {
    type DecrypterError = CryptoError;
    fn decrypt(&self, encrypted_entry: &EncryptedEntry) -> Result<InputEntry, Self::DecrypterError> {
        todo!()
    }
}

/// 随机数 可暴露，加密解密使用
pub type AesNonce = GenericArray<u8, U12>;



#[cfg(test)]
mod test {
    use aes_gcm::aead::consts::U12;
    use aes_gcm::aead::generic_array::GenericArray;
    use aes_gcm::aes::Aes256;
    use aes_gcm::{aead::{Aead, AeadCore, KeyInit, OsRng}, Aes256Gcm, AesGcm, Key, Nonce};
    use anyhow::anyhow;
    use base64ct::{Base64, Encoding};
    use std::str;

    #[test]
    fn test_nonce_ser_de_ser(){
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let vec = nonce.to_vec();
        let nb64 = Base64::encode_string(&vec);
        let vec1 = Base64::decode_vec(&nb64).unwrap();
        let new_nonce:  GenericArray<u8, U12> = Nonce::clone_from_slice(&vec1);
        println!("new_nonce: {:?}", new_nonce);
        println!("nonce: {:?}", new_nonce);
    }


    #[test]
    fn test_nonce() -> Result<(), Box<dyn std::error::Error>> {
        // 示例密钥（实际应用中应从安全来源获取）
        let key = Aes256Gcm::generate_key(&mut OsRng);
        let cipher: AesGcm<Aes256, U12> = Aes256Gcm::new(&key);

        // ---------------------- 生成 Nonce ----------------------
        let nonce: GenericArray<u8, U12> = Aes256Gcm::generate_nonce(&mut OsRng);
        println!("原始 Nonce (十六进制): {:02x?}", nonce.as_slice());

        // ---------------------- 序列化为字符串 ----------------------
        // 方法1: Base64 编码（推荐，紧凑格式）
        let nonce_base64 = Base64::encode_string(&nonce);
        println!("Base64 编码: {}", nonce_base64);

        // ---------------------- 从字符串反序列化 ----------------------
        // 从 Base64 还原
        let decoded_base64 = Base64::decode_vec(&nonce_base64)?;
        let nonce_from_base64: GenericArray<u8, U12> = Nonce::clone_from_slice(&decoded_base64);
        println!("从 Base64 还原: {:02x?}", nonce_from_base64.as_slice());

        // ---------------------- 验证还原的 Nonce ----------------------
        let plaintext = "测试数据";

        // 使用原始 Nonce 加密
        let ciphertext = cipher.encrypt(&nonce, plaintext.as_ref()).unwrap();

        // 使用还原的 Nonce 解密
        let decrypted = cipher.decrypt(&nonce_from_base64, ciphertext.as_ref()).unwrap();
        let decrypted_text = str::from_utf8(&decrypted)?;

        println!("\n解密验证: {}", decrypted_text);
        assert_eq!(plaintext, decrypted_text);

        Ok(())
    }

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