use aes::cipher::{BlockDecrypt, BlockEncrypt, KeyIvInit};
use aes::{Aes256, Block};
use rand::RngCore;
use std::path::Path;
use std::{fs, io};

type Aes256Enc = aes::cipher::generic_array::GenericArray<u8, aes::cipher::typenum::U32>;

pub struct CryptoManager {
    key: Aes256Enc,
}

impl CryptoManager {
    pub fn new(key_path: &Path) -> io::Result<Self> {
        let key = if key_path.exists() {
            fs::read(key_path)?
        } else {
            let mut key = vec![0u8; 32];
            rand::thread_rng().fill_bytes(&mut key);
            fs::write(key_path, &key)?;
            key
        };

        Ok(CryptoManager {
            key: Aes256Enc::clone_from_slice(&key),
        })
    }

    pub fn encrypt(&self, text: &str) -> io::Result<String> {
        let cipher = Aes256::new(&self.key);
        let mut blocks = Vec::new();

        for chunk in text.as_bytes().chunks(16) {
            let mut block = Block::<Aes256>::default();
            block[..chunk.len()].copy_from_slice(chunk);
            cipher.encrypt_block(&mut block);
            blocks.extend_from_slice(block.as_slice());
        }

        Ok(base64::encode(&blocks))
    }

    pub fn decrypt(&self, encrypted: &str) -> io::Result<String> {
        let cipher = Aes256::new(&self.key);
        let data =
            base64::decode(encrypted).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let mut blocks = Vec::new();
        for chunk in data.chunks(16) {
            let mut block = Block::<Aes256>::clone_from_slice(chunk);
            cipher.decrypt_block(&mut block);
            blocks.extend_from_slice(block.as_slice());
        }

        String::from_utf8(blocks).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}
