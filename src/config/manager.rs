use super::{Config, ModelConfig, PromptConfig};
use crate::config::CryptoManager;
use dirs;
use std::path::PathBuf;
use std::{fs, io};

pub struct ConfigManager {
    config: Config,
    config_path: PathBuf,
    crypto_manager: CryptoManager,
}

impl ConfigManager {
    pub fn new() -> io::Result<Self> {
        let config_dir = dirs::home_dir().unwrap_or_default().join(".tmchat");

        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }

        let config_path = config_dir.join("config.json");
        let config = if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            serde_json::from_str(&content).unwrap_or_else(|_| Config::new())
        } else {
            Config::new()
        };

        let crypto_manager = CryptoManager::new(&config_dir.join("aes_key.bin"))?;

        Ok(ConfigManager {
            config,
            config_path,
            crypto_manager,
        })
    }

    pub fn save(&self) -> io::Result<()> {
        let content = serde_json::to_string_pretty(&self.config)?;
        fs::write(&self.config_path, content)
    }

    pub fn set_model(&mut self, name: String, mut config: ModelConfig) -> io::Result<()> {
        // Encrypt API key before saving, if present
        if let Some(api_key) = config.api_key {
            config.api_key = Some(self.crypto_manager.encrypt(&api_key)?);
        }
        // If there's no default model, set the newly added one as default
        if self.config.default_model.is_none() {
            self.config.default_model = Some(name.clone());
        }
        self.config.models.insert(name, config);
        self.save()
    }

    pub fn set_prompt(&mut self, name: String, config: PromptConfig) -> io::Result<()> {
        // If there's no default prompt, set the newly added one as default
        if self.config.default_prompt.is_none() {
            self.config.default_prompt = Some(name.clone());
        }
        self.config.prompts.insert(name, config);
        self.save()``
    }

    pub fn get_model(&self, name: &str) -> Option<ModelConfig> {
        self.config.models.get(name).cloned().map(|mut config| {
            // Decrypt API key when retrieving, if present
            if let Some(api_key) = config.api_key {
                config.api_key = Some(self.crypto_manager.decrypt(&api_key).unwrap_or_default());
            }
            config
        })
    }

    pub fn get_prompt(&self, name: &str) -> Option<&PromptConfig> {
        self.config.prompts.get(name)
    }

    pub fn set_default_model(&mut self, name: String) -> io::Result<()> {
        self.config.default_model = Some(name);
        self.save()
    }

    pub fn set_default_prompt(&mut self, name: String) -> io::Result<()> {
        self.config.default_prompt = Some(name);
        self.save()
    }

    pub fn delete_model(&mut self, name: &str) -> io::Result<()> {
        self.config.models.remove(name);
        if self.config.default_model.as_deref() == Some(name) {
            self.config.default_model = None;
        }
        self.save()
    }

    pub fn delete_prompt(&mut self, name: &str) -> io::Result<()> {
        self.config.prompts.remove(name);
        // If the deleted prompt was the default, clear the default
        if self.config.default_prompt.as_deref() == Some(name) {
            self.config.default_prompt = None;
        }
        self.save()
    }

    pub fn list_models(&self) -> Vec<String> {
        self.config.models.keys().cloned().collect()
    }

    pub fn list_prompts(&self) -> Vec<String> {
        self.config.prompts.keys().cloned().collect()
    }

    pub fn get_default_model(&self) -> Option<String> {
        self.config.default_model.clone()
    }

    pub fn get_default_prompt(&self) -> Option<String> {
        self.config.default_prompt.clone()
    }
}
