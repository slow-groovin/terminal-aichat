use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};
use base64::{Engine, engine::general_purpose};
use rand::RngCore;
use std::path::Path;
use std::{fs, io};

pub struct CryptoManager {
    cipher: Aes256Gcm,
}

impl CryptoManager {
    pub fn new(key_path: &Path) -> io::Result<Self> {
        let key_bytes = if key_path.exists() {
            fs::read(key_path)?
        } else {
            let mut key = vec![0u8; 32];
            rand::rng().fill_bytes(&mut key);
            fs::write(key_path, &key)?;
            key
        };

        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);

        Ok(CryptoManager { cipher })
    }

    pub fn encrypt(&self, text: &str) -> io::Result<String> {
        // 生成随机 nonce
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        // 加密
        let ciphertext = self
            .cipher
            .encrypt(&nonce, text.as_bytes()) //
            .map_err(|e| {
                io::Error::new(io::ErrorKind::Other, format!("Encryption failed: {}", e))
            })?;

        // 将 nonce 和 ciphertext 合并后编码
        let mut result = nonce.to_vec();
        result.extend_from_slice(&ciphertext);

        Ok(general_purpose::STANDARD.encode(&result))
    }

    pub fn decrypt(&self, encrypted: &str) -> io::Result<String> {
        // 解码
        let data = general_purpose::STANDARD
            .decode(encrypted)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        if data.len() < 12 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "数据太短"));
        }

        // 分离 nonce 和 ciphertext
        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        // 解密
        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

        String::from_utf8(plaintext).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}
