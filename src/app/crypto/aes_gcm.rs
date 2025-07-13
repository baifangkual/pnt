//! 使用 aes_gcm 库进行加密和解密

use crate::app::crypto::{Decrypter, Encrypter};
use crate::app::entry::{EncryptedEntry, InputEntry, ValidEntry};
use crate::app::errors::CryptoError;
use aes_gcm::aead::OsRng;
use aes_gcm::aead::consts::U12;
use aes_gcm::aead::generic_array::GenericArray;
use aes_gcm::aes::Aes256;
use aes_gcm::{
    Aes256Gcm, AesGcm, Key, Nonce,
    aead::{Aead, AeadCore, KeyInit},
};
use anyhow::Result;
use base64ct::{Base64, Encoding};

/// 使用 aes256gcm 实现对 string 的加密解密，
/// 该实现中，同明文在不同次加密时会被加密为不同密文，
/// 同明文对应的不同密文解密可得到同明文
///
/// 使用Box引用，因为AesGcm占1000多字节（clippy分析），在Event enum中第二大的
/// Crossterm(CrosstermEvent) 变体只占 24 bytes，用指针优化Event枚举大小
struct StrAes256GcmEncrypter(Box<AesGcm<Aes256, U12>>);

impl StrAes256GcmEncrypter {
    fn from_key(key: [u8; 32]) -> Result<Self> {
        let gcm = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
        Ok(Self(Box::from(gcm)))
    }
    #[cfg(test)]
    fn from_random_key() -> StrAes256GcmEncrypter {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        Self(Box::from(Aes256Gcm::new(&key)))
    }
}

impl Encrypter<&str, String> for StrAes256GcmEncrypter {
    type EncrypterError = CryptoError;
    fn encrypt(&self, plaintext: &str) -> Result<String, Self::EncrypterError> {
        // aes256使用12字节nonce
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let cipher = self
            .0
            .encrypt(&nonce, plaintext.as_bytes())
            .map_err(CryptoError::Encrypt)?;
        let s_n = nonce.as_slice();
        if s_n.len() != 12 {
            // 饱和校验
            return Err(CryptoError::InvalidNonceLength);
        }
        let mut vec = Vec::with_capacity(12 + plaintext.len());
        vec.extend_from_slice(s_n);
        vec.extend_from_slice(&cipher);
        Ok(Base64::encode_string(&vec))
    }
}
impl Decrypter<&str, String> for StrAes256GcmEncrypter {
    type DecrypterError = CryptoError;
    fn decrypt(&self, ciphertext: &str) -> Result<String, Self::DecrypterError> {
        // 前12字节为nonce
        Base64::decode_vec(ciphertext)
            .map_err(|_| CryptoError::DecodeCiphertext)
            .and_then(|vec| {
                let nonce: GenericArray<u8, U12> = Nonce::clone_from_slice(&vec[..12]);
                let vec_utf8 = self.0.decrypt(&nonce, &vec[12..]).map_err(CryptoError::Decrypt)?;
                String::from_utf8(vec_utf8).map_err(|_| CryptoError::DecodeNonce)
            })
    }
}

/// Entry 的 部分秘密字段的加密解密器
pub struct EntryAes256GcmSecretEncrypter {
    inner_enc: StrAes256GcmEncrypter,
}
impl EntryAes256GcmSecretEncrypter {
    pub fn from_key(key: [u8; 32]) -> Result<EntryAes256GcmSecretEncrypter> {
        Ok(Self {
            inner_enc: StrAes256GcmEncrypter::from_key(key)?,
        })
    }
    #[cfg(test)]
    fn from_random_key() -> EntryAes256GcmSecretEncrypter {
        Self {
            inner_enc: StrAes256GcmEncrypter::from_random_key(),
        }
    }
}

impl Encrypter<&InputEntry, ValidEntry> for EntryAes256GcmSecretEncrypter {
    type EncrypterError = CryptoError;
    fn encrypt(&self, input_entry: &InputEntry) -> Result<ValidEntry, Self::EncrypterError> {
        // 加密敏感字段
        let cipher_username = self.inner_enc.encrypt(&input_entry.username)?;
        let cipher_passwd = self.inner_enc.encrypt(&input_entry.password)?;
        Ok(ValidEntry {
            about: input_entry.about.clone(),
            notes: if input_entry.notes.is_empty() {
                None
            } else {
                Some(input_entry.notes.clone())
            },
            encrypted_username: cipher_username,
            encrypted_password: cipher_passwd,
        })
    }
}
impl Decrypter<&EncryptedEntry, InputEntry> for EntryAes256GcmSecretEncrypter {
    type DecrypterError = CryptoError;
    fn decrypt(&self, encrypted_entry: &EncryptedEntry) -> Result<InputEntry, Self::DecrypterError> {
        let username = self.inner_enc.decrypt(&encrypted_entry.encrypted_username)?;
        let password = self.inner_enc.decrypt(&encrypted_entry.encrypted_password)?;
        Ok(InputEntry {
            about: encrypted_entry.about.clone(),
            notes: if let Some(desc) = &encrypted_entry.notes {
                desc.clone()
            } else {
                String::new()
            },
            username,
            password,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::app::crypto::aes_gcm::{EntryAes256GcmSecretEncrypter, StrAes256GcmEncrypter};
    use crate::app::crypto::{Decrypter, Encrypter};
    use crate::app::entry::{EncryptedEntry, InputEntry};
    use chrono::DateTime;

    #[test]
    fn test_encrypt_decrypt_entry() {
        let encrypter = EntryAes256GcmSecretEncrypter::from_random_key();
        let u_input = InputEntry {
            about: "name".to_owned(),
            notes: String::new(),
            username: "def".to_owned(),
            password: "abc".to_owned(),
        };
        let v_e = encrypter.encrypt(&u_input).unwrap();
        let enc_entry = EncryptedEntry {
            id: 123,
            about: v_e.about,
            notes: v_e.notes,
            encrypted_username: v_e.encrypted_username,
            encrypted_password: v_e.encrypted_password,
            created_time: DateTime::default(),
            updated_time: DateTime::default(),
        };
        let entry = encrypter.decrypt(&enc_entry).unwrap();
        assert_eq!(u_input.about, entry.about);
        assert_eq!(u_input.notes, entry.notes);
        assert_eq!(u_input.password, entry.password);
        assert_eq!(u_input.username, entry.username);
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
}
