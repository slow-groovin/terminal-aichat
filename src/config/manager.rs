use super::Config;
use super::crypto::CryptoManager;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// 配置文件管理器 - 只负责I/O操作
pub struct ConfigManager {
    config_path: PathBuf,
    crypto: CryptoManager,
}

impl ConfigManager {
    pub fn new(config_dir: &Path) -> io::Result<Self> {
        if !config_dir.exists() {
            fs::create_dir_all(config_dir)?;
        }

        Ok(Self {
            config_path: config_dir.join("config.json"),
            crypto: CryptoManager::new(&config_dir.join("aes_key.bin"))?,
        })
    }

    /// 从文件加载配置
    pub fn load(&self) -> io::Result<Config> {
        if !self.config_path.exists() {
            return Ok(Config::default());
        }

        let content = fs::read_to_string(&self.config_path)?;
        let mut config: Config =
            serde_json::from_str(&content).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        // 解密所有API key
        self.decrypt_config(&mut config)?;

        Ok(config)
    }

    /// 保存配置到文件
    pub fn save(&self, config: &Config) -> io::Result<()> {
        let mut config = config.clone();

        // 加密所有API key
        self.encrypt_config(&mut config)?;

        let content = serde_json::to_string_pretty(&config)?;
        fs::write(&self.config_path, content)
    }

    /// 配置文件是否存在
    pub fn exists(&self) -> bool {
        self.config_path.exists()
    }

    /// 加密配置中的所有API key
    fn encrypt_config(&self, config: &mut Config) -> io::Result<()> {
        for model in config.models.values_mut() {
            if let Some(key) = &model.api_key {
                if !key.is_empty() {
                    model.api_key = Some(self.crypto.encrypt(key)?);
                }
            }
        }
        Ok(())
    }

    /// 解密配置中的所有API key
    fn decrypt_config(&self, config: &mut Config) -> io::Result<()> {
        for model in config.models.values_mut() {
            if let Some(key) = &model.api_key {
                if !key.is_empty() {
                    model.api_key = Some(self.crypto.decrypt(key)?);
                }
            }
        }
        Ok(())
    }
}
