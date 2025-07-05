//! 使用 aes_gcm 库进行加密和解密

use crate::app::crypto::{Decrypter, Encrypter, MainPwdVerifier};
use crate::app::entry::{EncryptedEntry, InputEntry, ValidEntry};
use crate::app::errors::CryptoError;
use aes_gcm::aead::consts::U12;
use aes_gcm::aead::generic_array::GenericArray;
use aes_gcm::aes::Aes256;
use aes_gcm::{aead::{Aead, AeadCore, KeyInit}, Aes256Gcm, AesGcm, Key, Nonce};
use aes_gcm::aead::OsRng;
use anyhow::Result;
use base64ct::{Base64, Encoding};

/// 使用 aes256gcm 实现对 string 的加密解密
/// 该实现中，同明文在不同次加密时会被加密为不同密文
/// 同明文对应的不同密文解密可得到同明文
struct StrAes256GcmEncrypter(AesGcm<Aes256, U12>);

impl StrAes256GcmEncrypter {
    fn from_key(key: [u8; 32]) -> Result<Self> {
        let gcm = aes_gcm::Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
        Ok(Self(gcm))
    }
    fn from_random_key() -> StrAes256GcmEncrypter {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        Self (aes_gcm::Aes256Gcm::new(&key))
    }
}

impl Encrypter<&str, String> for StrAes256GcmEncrypter {
    type EncrypterError = CryptoError;
    fn encrypt(&self, plaintext: &str) -> Result<String, Self::EncrypterError> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let cipher = self.0
            .encrypt(&nonce, plaintext.as_ref())
            .map_err(|e| CryptoError::Encrypt(e))?;
        let nb64_enc = format!("{}:{}",Base64::encode_string(&nonce.to_vec()), Base64::encode_string(&cipher) );
        Ok(nb64_enc)
    }
}
impl Decrypter<&str, String> for StrAes256GcmEncrypter {
    type DecrypterError = CryptoError;
    fn decrypt(&self, ciphertext: &str) -> Result<String, Self::DecrypterError> {
        let Some((nb64, enc)) = ciphertext.split_once(":") else {
            return Err(CryptoError::CiphertextSplit)
        };
        let nonce_vec = Base64::decode_vec(nb64).map_err(|_| CryptoError::DecodeNonce)?;
        let enc_vec = Base64::decode_vec(enc).map_err(|_| CryptoError::DecodeCiphertext)?;
        let nonce: GenericArray<u8, U12> = Nonce::clone_from_slice(&nonce_vec);
        let vec_utf8 = self.0.decrypt(&nonce, enc_vec.as_ref())
            .map_err(|e| CryptoError::Decrypt(e))?;
        Ok(String::from_utf8(vec_utf8).map_err(|_| CryptoError::DecodeNonce)?)
    }
}

/// Entry 的 部分秘密字段的加密解密器
pub struct EntryAes256GcmSecretEncrypter {
    inner_enc: StrAes256GcmEncrypter,
}
impl EntryAes256GcmSecretEncrypter {
    pub fn from_key(key: [u8; 32]) -> Result<EntryAes256GcmSecretEncrypter> {
        Ok(Self { inner_enc: StrAes256GcmEncrypter::from_key(key)? })
    }

    fn from_random_key() -> EntryAes256GcmSecretEncrypter {
        Self { inner_enc: StrAes256GcmEncrypter::from_random_key() }
    }
}

impl Encrypter<&InputEntry, ValidEntry> for EntryAes256GcmSecretEncrypter {
    type EncrypterError = CryptoError;
    fn encrypt(&self, input_entry: &InputEntry) -> Result<ValidEntry, Self::EncrypterError> {
        // 加密敏感字段
        let cipher_identity = self.inner_enc
            .encrypt(&input_entry.identity)?;
        let cipher_passwd = self.inner_enc
            .encrypt(&input_entry.password)?;
        Ok(ValidEntry {
            name: input_entry.name.clone(),
            description: if input_entry.description.is_empty() {
                None
            } else {
                Some(input_entry.description.clone())
            },
            encrypted_identity: cipher_identity,
            encrypted_password: cipher_passwd,
        })
    }
}
impl Decrypter<&EncryptedEntry, InputEntry> for EntryAes256GcmSecretEncrypter {
    type DecrypterError = CryptoError;
    fn decrypt(&self, encrypted_entry: &EncryptedEntry) -> Result<InputEntry, Self::DecrypterError> {
        let identity = self.inner_enc
            .decrypt(&encrypted_entry.encrypted_identity)?;
        let password = self.inner_enc.decrypt(&encrypted_entry.encrypted_password)?;
        Ok(InputEntry {
            name: encrypted_entry.name.clone(),
            description: if let Some(desc) = &encrypted_entry.description {desc.clone()} else {String::new()},
            identity,
            password,
        })
    }
}

#[cfg(test)]
mod test {
    use aes_gcm::aead::consts::U12;
    use aes_gcm::aead::generic_array::GenericArray;
    use aes_gcm::aes::Aes256;
    use aes_gcm::{aead::{Aead, AeadCore, KeyInit, OsRng}, Aes256Gcm, AesGcm, Key, Nonce};
    use anyhow::anyhow;
    use base64ct::{Base64, Encoding};
    use std::str;
    use chrono::DateTime;
    use crate::app::crypto::aes_gcm::{EntryAes256GcmSecretEncrypter, StrAes256GcmEncrypter};
    use crate::app::crypto::{Decrypter, Encrypter};
    use crate::app::entry::{EncryptedEntry, InputEntry};

    #[test]
    fn test_encrypt_decrypt_entry() {
        let encrypter = EntryAes256GcmSecretEncrypter::from_random_key();
        let u_input = InputEntry {
            name: "name".to_owned(),
            description: String::new(),
            identity: "def".to_owned(),
            password: "abc".to_owned(),
        };
        let v_e = encrypter.encrypt(&u_input).unwrap();
        let enc_entry = EncryptedEntry {
            id: 123,
            name: v_e.name,
            description: v_e.description,
            encrypted_identity: v_e.encrypted_identity,
            encrypted_password: v_e.encrypted_password,
            created_at: DateTime::default(),
            updated_at: DateTime::default(),
        };
        let entry = encrypter.decrypt(&enc_entry).unwrap();
        assert_eq!(u_input.name, entry.name);
        assert_eq!(u_input.description, entry.description);
        assert_eq!(u_input.password, entry.password);
        assert_eq!(u_input.identity, entry.identity);
    }


    #[test]
    fn test_str_aes256_gcm_impl() {
        let encrypter = StrAes256GcmEncrypter::from_random_key();
        let plaintext = "hello world";
        let cip = encrypter.encrypt(plaintext).unwrap();
        // println!("{}", cip);
        let plain2 = encrypter.decrypt(&cip).unwrap();
        // println!("{}", plain2);
        assert_eq!(plaintext, plain2);

    }

    // #[test]
    // fn test_nonce_ser_de_ser() {
    //     let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    //     let vec = nonce.to_vec();
    //     let nb64 = Base64::encode_string(&vec);
    //     let vec1 = Base64::decode_vec(&nb64).unwrap();
    //     let new_nonce: GenericArray<u8, U12> = Nonce::clone_from_slice(&vec1);
    //     println!("new_nonce: {:?}", new_nonce);
    //     println!("nonce: {:?}", new_nonce);
    // }

    // #[test]
    // fn test_nonce() -> Result<(), Box<dyn std::error::Error>> {
    //     // 示例密钥（实际应用中应从安全来源获取）
    //     let key = Aes256Gcm::generate_key(&mut OsRng);
    //     let cipher: AesGcm<Aes256, U12> = Aes256Gcm::new(&key);
    //
    //     // ---------------------- 生成 Nonce ----------------------
    //     let nonce: GenericArray<u8, U12> = Aes256Gcm::generate_nonce(&mut OsRng);
    //     println!("原始 Nonce (十六进制): {:02x?}", nonce.as_slice());
    //
    //     // ---------------------- 序列化为字符串 ----------------------
    //     // 方法1: Base64 编码（推荐，紧凑格式）
    //     let nonce_base64 = Base64::encode_string(&nonce);
    //     println!("Base64 编码: {}", nonce_base64);
    //
    //     // ---------------------- 从字符串反序列化 ----------------------
    //     // 从 Base64 还原
    //     let decoded_base64 = Base64::decode_vec(&nonce_base64)?;
    //     let nonce_from_base64: GenericArray<u8, U12> = Nonce::clone_from_slice(&decoded_base64);
    //     println!("从 Base64 还原: {:02x?}", nonce_from_base64.as_slice());
    //
    //     // ---------------------- 验证还原的 Nonce ----------------------
    //     let plaintext = "测试数据";
    //
    //     // 使用原始 Nonce 加密
    //     let ciphertext = cipher.encrypt(&nonce, plaintext.as_ref()).unwrap();
    //
    //     // 使用还原的 Nonce 解密
    //     let decrypted = cipher.decrypt(&nonce_from_base64, ciphertext.as_ref()).unwrap();
    //     let decrypted_text = str::from_utf8(&decrypted)?;
    //
    //     println!("\n解密验证: {}", decrypted_text);
    //     assert_eq!(plaintext, decrypted_text);
    //
    //     Ok(())
    // }

    // #[test]
    // fn test_aes_gcm() -> anyhow::Result<()> {
    //     // 示例明文
    //     let plaintext = "这是需要加密的绝密数据!";
    //     let my_key = "my_key";
    //     let salt = "my_salt----".as_bytes();
    //     let mut p_hash = [0u8; 32];
    //     argon2::Argon2::default()
    //         .hash_password_into(my_key.as_bytes(), salt, &mut p_hash)
    //         .map_err(|e| anyhow!(e))?;
    //     // ---------------------- 加密部分 ----------------------
    //     // 1. 生成随机密钥 (256位 = 32字节)
    //     // let key = Aes256Gcm::generate_key(&mut OsRng);
    //     let key = Key::<Aes256Gcm>::from_slice(&p_hash);
    //     // 2. 初始化加密器
    //     let cipher = Aes256Gcm::new(&key);
    //
    //     // 3. 生成随机 nonce (12字节 - GCM标准长度)
    //     let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    //
    //     // 4. 加密数据（自动附加认证标签）
    //     let ciphertext = cipher
    //         .encrypt(&nonce, plaintext.as_ref())
    //         .expect("加密失败");
    //
    //     println!("加密成功！");
    //     println!("密钥 (Base64): {}", Base64::encode_string(&key));
    //     println!("Nonce (Base64): {}", Base64::encode_string(&nonce));
    //     println!("密文 (Base64): {}", Base64::encode_string(&ciphertext));
    //
    //     // ---------------------- 解密部分 ----------------------
    //     // 1. 重新创建加密器（实际应用中需从安全存储获取密钥）
    //     let decipher = Aes256Gcm::new(&key);
    //
    //     // 2. 解密数据（同时验证认证标签）
    //     let decrypted_data = decipher
    //         .decrypt(&nonce, ciphertext.as_ref())
    //         .expect("解密失败 - 认证标签无效或数据篡改！");
    //
    //     // 3. 将解密后的字节转为字符串
    //     let decrypted_text = str::from_utf8(&decrypted_data)?;
    //
    //     println!("\n解密成功！");
    //     println!("原始明文: {}", plaintext);
    //     println!("解密结果: {}", decrypted_text);
    //
    //     Ok(())
    // }
}