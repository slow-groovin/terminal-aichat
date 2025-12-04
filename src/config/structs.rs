use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelConfig {
    pub model_name: Option<String>,
    pub base_url: Option<String>,
    pub api_key: Option<String>, // This will be encrypted in the config file
    pub temperature: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PromptConfig {
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub models: HashMap<String, ModelConfig>,
    pub prompts: HashMap<String, PromptConfig>,
    #[serde(rename = "default-model")]
    pub default_model: Option<String>,
    #[serde(rename = "default-prompt")]
    pub default_prompt: Option<String>,
    #[serde(rename = "disable-stream")]
    pub disable_stream: bool,
    pub pure: bool,
    pub verbose: bool,
}

impl ModelConfig {
    /// 合并配置,优先使用self的值
    pub fn merge_with(self, base: &ModelConfig) -> Self {
        Self {
            model_name: self.model_name.or_else(|| base.model_name.clone()),
            base_url: self.base_url.or_else(|| base.base_url.clone()),
            api_key: self.api_key.or_else(|| base.api_key.clone()),
            temperature: self.temperature.or(base.temperature),
        }
    }
}

impl PromptConfig {
    pub fn merge_with(self, base: &PromptConfig) -> Self {
        Self {
            content: if self.content.is_empty() {
                base.content.clone()
            } else {
                self.content
            },
        }
    }
}

impl Config {
    pub fn default() -> Self {
        Config {
            models: HashMap::new(),
            prompts: HashMap::new(),
            default_model: None,
            default_prompt: None,
            disable_stream: false,
            pure: false,
            verbose: false,
        }
    }
}
