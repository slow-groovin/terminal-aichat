use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelConfig {
    pub model_name: Option<String>,
    pub base_url: Option<String>,
    pub api_key: Option<String>, // This will be encrypted in the config file
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PromptConfig {
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub models: HashMap<String, ModelConfig>,
    pub prompts: HashMap<String, PromptConfig>,
    #[serde(rename = "default-model")]
    pub default_model: Option<String>,
    #[serde(rename = "default-prompt")]
    pub default_prompt: Option<String>,
    #[serde(rename = "disable-stream")]
    pub disable_stream: Option<bool>,
    pub pure: Option<bool>,
    pub verbose: Option<bool>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            models: HashMap::new(),
            prompts: HashMap::new(),
            default_model: None,
            default_prompt: None,
            disable_stream: Some(false),
            pure: Some(false),
            verbose: Some(false),
        }
    }
}
