use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng}, Aes256Gcm, Error, Key
};

fn main() -> Result<(),Error> {
    // 生成随机密钥
    
    // let key = Aes256Gcm::generate_key(OsRng);
    // 从密码派生密钥
    let key_bytes = derive_key_from_password("my_secret_password");
    let key=Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(&key);
    
    // 要加密的数据
    let plaintext = b"Hello, AES-GCM!";
    
    // 生成随机 nonce
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    
    // 加密
    let ciphertext = cipher.encrypt(&nonce, plaintext.as_ref())?;
    println!("加密后: {:?}", ciphertext);
    
    // 解密
    let decrypted = cipher.decrypt(&nonce, ciphertext.as_ref())?;
    println!("解密后: {}", String::from_utf8(decrypted).unwrap());
    
    Ok(())
}
use sha2::{Sha256, Digest};
fn derive_key_from_password(password: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.finalize().into()
}