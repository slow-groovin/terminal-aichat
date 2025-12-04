use super::{Config, ModelConfig, PromptConfig};
/// 配置构建器 - 用于修改配置
pub struct ConfigBuilder {
    config: Config,
}

impl ConfigBuilder {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// 设置或更新模型配置
    pub fn set_model(mut self, name: String, model: ModelConfig) -> Self {
        let merged = if let Some(existing) = self.config.models.get(&name) {
            model.merge_with(existing)
        } else {
            model
        };

        // 如果没有默认模型,设为默认
        if self.config.default_model.is_none() {
            self.config.default_model = Some(name.clone());
        }

        self.config.models.insert(name, merged);
        self
    }

    /// 设置或更新提示配置
    pub fn set_prompt(mut self, name: String, prompt: PromptConfig) -> Self {
        let merged = if let Some(existing) = self.config.prompts.get(&name) {
            prompt.merge_with(existing)
        } else {
            prompt
        };

        // 如果没有默认提示,设为默认
        if self.config.default_prompt.is_none() {
            self.config.default_prompt = Some(name.clone());
        }

        self.config.prompts.insert(name, merged);
        self
    }

    /// 设置默认模型
    pub fn set_default_model(mut self, name: String) -> Self {
        self.config.default_model = Some(name);
        self
    }

    /// 设置默认提示
    pub fn set_default_prompt(mut self, name: String) -> Self {
        self.config.default_prompt = Some(name);
        self
    }

    /// 初始化默认配置
    pub fn with_defaults(self) -> Self {
        let default_model_name = "sample_model_gpt".to_string();
        let default_prompt_name = "sample_prompt".to_string();

        self.set_model(
            default_model_name.clone(),
            ModelConfig {
                model_name: Some("gpt-5-mini".to_string()),
                base_url: Some("https://api.openai.com/v1".to_string()),
                api_key: None,
                temperature: None,
            },
        )
        .set_prompt(
            default_prompt_name.clone(),
            PromptConfig {
                content: r#"You are a terminal assistant. 
You are giving help to user in the terminal.
Give concise responses whenever possible.
Because of terminal cannot render markdown, DO NOT contain any markdown syntax(`,```, #, ...) in your response, use plain text only.
"#
                .to_string(),
            },
        )
        .set_default_model(default_model_name)
        .set_default_prompt(default_prompt_name)
    }

    /// 构建最终配置
    pub fn build(self) -> Config {
        self.config
    }
}
