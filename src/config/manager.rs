use super::{Config, ModelConfig, PromptConfig};
use crate::cli::structs::Cli;
use crate::config::CryptoManager;
use clap::Parser;
use crossterm::style::Stylize;
use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
};
use dirs;
use std::collections::HashMap;
use std::path::PathBuf;
use std::{fs, io};

pub struct ConfigManager {
    ///merged config (file+cli)
    config: Config,
    file_config: Config,
    config_path: PathBuf,
    crypto_manager: CryptoManager,
}

impl ConfigManager {
    pub fn load() -> io::Result<Self> {
        let config_dir = dirs::home_dir().unwrap_or_default().join(".terminal-aichat");

        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }

        let config_path = config_dir.join("config.json");
        let file_config = if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            serde_json::from_str(&content).unwrap_or_else(|_| Config::new())
        } else {
            Config::new()
        };

        let merged_config = resolve_configuration(Cli::parse(), file_config.clone());
        let crypto_manager = CryptoManager::new(&config_dir.join("aes_key.bin"))?;

        Ok(ConfigManager {
            config: merged_config,
            file_config,
            config_path,
            crypto_manager,
        })
    }

    /// Checks if the configuration file already exists.
    pub fn config_file_exists(&self) -> bool {
        self.config_path.exists()
    }

    /// Saves the current configuration to the file.
    pub fn save(&self) -> io::Result<()> {
        let content = serde_json::to_string_pretty(&self.file_config)?;
        fs::write(&self.config_path, content)
    }

    pub fn try_initialize_default_configs_and_save(&mut self) -> io::Result<()> {
        if self.config_file_exists() {
            return Ok(());
        } else {
            let _ = self.initialize_default_configs();
            self.save()?;
            self.config = resolve_configuration(Cli::parse(), self.file_config.clone());
            Ok(())
        }
    }

    /// Initializes default model and prompt configurations if they don't exist.
    pub fn initialize_default_configs(&mut self) -> io::Result<(String, String)> {
        // Define a default model configuration
        let default_model_name = "sample_model_gpt".to_string();
        let default_model_config = ModelConfig {
            model_name: Some("gpt-5-mini".to_string()), // Example default model
            base_url: Some("https://api.openai.com/v1".to_string()), // Example default base URL
            api_key: None,                              // API key can be set later or via environment variable
        };
        self.set_model(default_model_name.clone(), default_model_config)?;

        // Define a default prompt configuration
        let default_prompt_name = "sample_prompt".to_string();
        let default_prompt_content = r#"You are a terminal assistant. 
You are giving help to user in the terminal.
Do not use any Markdown syntax, use plain text only."#
            .to_string();
        let default_prompt_config = PromptConfig {
            content: default_prompt_content,
        };
        self.set_prompt(default_prompt_name.clone(), default_prompt_config)?;

        // Set the newly created model and prompt as default
        self.set_default_model(default_model_name.clone())?;
        self.set_default_prompt(default_prompt_name.clone())?;

        Ok((default_model_name, default_prompt_name))
    }

    pub fn set_model(&mut self, name: String, mut config: ModelConfig) -> io::Result<()> {
        let exist_model = self.get_model(&name);
        // Encrypt API key before saving, if present
        if let Some(api_key) = config.api_key {
            config.api_key = Some(self.crypto_manager.encrypt(&api_key)?);
        }

        //merge with exist config
        let config = merge_model(config, exist_model);
        // If there's no default model, set the newly added one as default
        if self.file_config.default_model.is_none() {
            self.file_config.default_model = Some(name.clone());
        }
        self.file_config.models.insert(name, config);
        self.save()
    }

    pub fn set_prompt(&mut self, name: String, config: PromptConfig) -> io::Result<()> {
        let exist_prompt = self.get_prompt(&name);
        let config = merge_prompt(config, exist_prompt);

        // If there's no default prompt, set the newly added one as default
        if self.file_config.default_prompt.is_none() {
            self.file_config.default_prompt = Some(name.clone());
        }

        self.file_config.prompts.insert(name, config);
        self.save()
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

    pub fn decrypt_api_key(&self, api_key: &str) -> Result<String, io::Error> {
        self.crypto_manager.decrypt(&api_key)
    }

    pub fn get_prompt(&self, name: &str) -> Option<&PromptConfig> {
        self.config.prompts.get(name)
    }

    pub fn set_default_model(&mut self, name: String) -> io::Result<()> {
        self.file_config.default_model = Some(name);
        self.save()
    }

    pub fn set_default_prompt(&mut self, name: String) -> io::Result<()> {
        self.file_config.default_prompt = Some(name);
        self.save()
    }

    pub fn delete_model(&mut self, name: &str) -> io::Result<()> {
        self.file_config.models.remove(name);
        if self.config.default_model.as_deref() == Some(name) {
            self.config.default_model = None;
        }
        self.save()
    }

    pub fn delete_prompt(&mut self, name: &str) -> io::Result<()> {
        self.file_config.prompts.remove(name);
        // If the deleted prompt was the default, clear the default
        if self.config.default_prompt.as_deref() == Some(name) {
            self.config.default_prompt = None;
        }
        self.save()
    }

    pub fn list_models(&self) {
        let _ = print_models_table(&self.file_config.models, &self.get_default_model_name(), |k| {
            self.decrypt_api_key(k)
        });
    }

    pub fn list_prompts(&self) {
        print_prompts_list(&self.file_config.prompts, self.get_default_prompt_name());
    }

    pub fn get_default_model_name(&self) -> Option<String> {
        self.config.default_model.clone()
    }

    pub fn get_default_prompt_name(&self) -> Option<String> {
        self.config.default_prompt.clone()
    }
}
fn merge_model(mut new_config: ModelConfig, base_config: Option<ModelConfig>) -> ModelConfig {
    if base_config.is_none() {
        return new_config;
    }
    let ModelConfig {
        api_key,
        base_url,
        model_name,
    } = base_config.unwrap();
    new_config.api_key = new_config.api_key.or(api_key);
    new_config.model_name = new_config.model_name.or(model_name);
    new_config.base_url = new_config.base_url.or(base_url);
    return new_config;
}

fn merge_prompt(mut new_config: PromptConfig, base_config: Option<&PromptConfig>) -> PromptConfig {
    if base_config.is_none() {
        return new_config;
    }
    new_config.content = if new_config.content.is_empty() {
        base_config.as_ref().unwrap().content.clone()
    } else {
        new_config.content
    };
    return new_config;
}

/**
 * 解析/解决配置, 优先级为：配置文件 < 参数
 */
fn resolve_configuration(cli: Cli, config: Config) -> Config {
    let mut resolved_config = config;

    // 1. Resolve Model Configuration
    let effective_model_name = cli.model.or(resolved_config.default_model);
    // 2. Resolve Prompt Configuration
    let effective_prompt_name = cli.prompt.or(resolved_config.default_prompt);

    resolved_config.default_prompt = effective_prompt_name;

    resolved_config.default_model = effective_model_name;

    // 3. Resolve Global Flags (disable_stream, pure, verbose)
    // Priority: CLI > Config File

    // disable_stream
    resolved_config.disable_stream = Some(cli.disable_stream || resolved_config.disable_stream.unwrap_or(false));

    // pure
    resolved_config.pure = Some(cli.pure || resolved_config.pure.unwrap_or(false));

    // verbose
    resolved_config.verbose = Some(cli.verbose || resolved_config.verbose.unwrap_or(false));

    resolved_config
}

pub fn print_models_table<F>(
    models: &HashMap<String, ModelConfig>,
    default_model_name: &Option<String>,
    decrypt_func: F,
) -> io::Result<()>
where
    F: Fn(&str) -> Result<String, std::io::Error>,
{
    let mut stdout = io::stdout();

    // 表头
    execute!(
        stdout,
        Print(
            "┌─────────────────────┬─────────────────────┬─────────────────────────────────────┬─────────────────┐\n"
        )
    )?;
    execute!(
        stdout,
        Print(
            "│ Name                │ Model Name          │ Base URL                            │ API Key         │\n"
        )
    )?;
    execute!(
        stdout,
        Print(
            "├─────────────────────┼─────────────────────┼─────────────────────────────────────┼─────────────────┤\n"
        )
    )?;

    // 数据行
    for (name, config) in models {
        let is_default = default_model_name.as_ref().map_or(false, |default| name == default);

        execute!(stdout, Print("│ "))?;

        // Name字段
        if is_default {
            execute!(stdout, SetForegroundColor(Color::Green))?;
        } else {
            execute!(stdout, SetForegroundColor(Color::DarkBlue))?;
        }
        let name_display = if name.len() > 19 {
            format!("{}...", &name[..16])
        } else {
            name.clone()
        };
        execute!(stdout, Print(format!("{:<19}", name_display)))?;
        execute!(stdout, ResetColor)?;
        execute!(stdout, Print(" │ "))?;

        // Model Name字段
        if is_default {
            execute!(stdout, SetForegroundColor(Color::Green))?;
        } else {
            execute!(stdout, SetForegroundColor(Color::DarkBlue))?;
        }
        let model_name = config.model_name.as_deref().unwrap_or("");
        let model_name_display = if model_name.len() > 19 {
            format!("{}...", &model_name[..16])
        } else {
            model_name.to_string()
        };
        execute!(stdout, Print(format!("{:<19}", model_name_display)))?;
        execute!(stdout, ResetColor)?;
        execute!(stdout, Print(" │ "))?;

        // Base URL字段
        if is_default {
            execute!(stdout, SetForegroundColor(Color::Green))?;
        } else {
            execute!(stdout, SetForegroundColor(Color::DarkBlue))?;
        }
        let base_url = config.base_url.as_deref().unwrap_or("");
        let base_url_display = if base_url.len() > 35 {
            format!("{}...", &base_url[..32])
        } else {
            base_url.to_string()
        };
        execute!(stdout, Print(format!("{:<35}", base_url_display)))?;
        execute!(stdout, ResetColor)?;
        execute!(stdout, Print(" │ "))?;

        // API Key字段
        if is_default {
            execute!(stdout, SetForegroundColor(Color::Green))?;
        } else {
            execute!(stdout, SetForegroundColor(Color::DarkBlue))?;
        }
        let api_key_display = match &config.api_key {
            Some(key) if !key.is_empty() => {
                let key = decrypt_func(key).unwrap();
                if key.len() <= 8 {
                    format!("{}***", &key[..key.len().min(4)])
                } else {
                    format!("{}...{}", &key[..4], &key[key.len() - 4..])
                }
            }
            _ => "".to_string(),
        };
        execute!(stdout, Print(format!("{:<15}", api_key_display)))?;
        execute!(stdout, ResetColor)?;
        execute!(stdout, Print(" │\n"))?;
    }

    // 表格底部
    execute!(
        stdout,
        Print(
            "└─────────────────────┴─────────────────────┴─────────────────────────────────────┴─────────────────┘\n"
        )
    )?;

        // 显示当前默认模型
    if let Some(default) = default_model_name {
        execute!(stdout, Print("Current default model: "))?;
        //TODO: 改为直接使用.green()而不是set方法
        // execute!(stdout, SetForegroundColor(Color::Green))?;
        execute!(stdout, Print(format!("{}\n", default.clone().green())))?;
        // execute!(stdout, ResetColor)?;
    }


    Ok(())
}

pub fn print_prompts_list(prompts: &HashMap<String, PromptConfig>, default_prompt_name: Option<String>) {
    for (k, v) in prompts {
        let default_text = match default_prompt_name.as_deref() {
            Some(_s) if _s == k => "(default)",
            _ => "",
        };
        println!("{}{}:", k.as_str().blue().bold(), default_text.green());
        println!(
            "{}\n{}\n{}\n",
            "```".blue().bold(),
            v.content.as_str(),
            "```".blue().bold()
        );
    }
}
