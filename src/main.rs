mod chat;
mod cli;
mod config;
mod crypto;

use clap::Parser;
use cli::{Cli, Commands, DeleteCommands, SetCommands, UseCommands};
use config::manager::ConfigManager;
use config::{ModelConfig, PromptConfig};
use crossterm::style::Stylize;

fn list_models(config_manager: &ConfigManager, verbose: bool) {
    for model in config_manager.list_models() {
        let is_default = config_manager
            .get_default_model()
            .map(|m| m == model)
            .unwrap_or(false);
        if verbose {
            if let Some(config) = config_manager.get_model(&model) {
                print!("  {}", model);
                if is_default {
                    print!(" {}", "(default)".green());
                }
                println!(
                    " (model: {}, base_url: {})",
                    config.model_name, config.base_url
                );
            }
        } else {
            print!("  {}", model);
            if is_default {
                println!(" {}", "(default)".green());
            } else {
                println!();
            }
        }
    }
}

fn list_prompts(config_manager: &ConfigManager) {
    for prompt in config_manager.list_prompts() {
        let is_default = config_manager
            .get_default_prompt()
            .map(|p| p == prompt)
            .unwrap_or(false);
        print!("  {}", prompt);
        if is_default {
            println!(" {}", "(default)".green());
        } else {
            println!();
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Initialize config manager
    let mut config_manager = ConfigManager::new()?;

    // Handle subcommands
    match &cli.command {
        Some(Commands::Set { config }) => match config {
            SetCommands::Model {
                name,
                base_url,
                model_name,
                api_key,
            } => {
                let api_key = api_key
                    .clone()
                    .or_else(|| std::env::var("OPENAI_API_KEY").ok())
                    .ok_or("API key is required")?;

                config_manager.set_model(
                    name.clone(),
                    ModelConfig {
                        base_url: base_url.clone(),
                        model_name: model_name.clone(),
                        api_key,
                    },
                )?;
                println!(
                    "{}",
                    format!("Model configuration '{}' has been set.", name).green()
                );
            }
            SetCommands::Prompt { name, content } => {
                config_manager.set_prompt(
                    name.clone(),
                    PromptConfig {
                        content: content.clone(),
                    },
                )?;
                println!(
                    "{}",
                    format!("Prompt configuration '{}' has been set.", name).green()
                );
            }
        },
        Some(Commands::Use { config }) => match config {
            UseCommands::Model { name } => {
                if config_manager.get_model(name).is_some() {
                    config_manager.set_default_model(name.clone())?;
                    println!(
                        "{}",
                        format!("Default model has been set to '{}'.", name).green()
                    );
                } else {
                    eprintln!(
                        "{}",
                        format!("Model configuration '{}' not found.", name).red()
                    );
                }
            }
            UseCommands::Prompt { name } => {
                if config_manager.get_prompt(name).is_some() {
                    config_manager.set_default_prompt(name.clone())?;
                    println!(
                        "{}",
                        format!("Default prompt has been set to '{}'.", name).green()
                    );
                } else {
                    eprintln!(
                        "{}",
                        format!("Prompt configuration '{}' not found.", name).red()
                    );
                }
            }
        },
        Some(Commands::Delete { config }) => match config {
            DeleteCommands::Model { name } => {
                config_manager.delete_model(name)?;
                println!(
                    "{}",
                    format!("Model configuration '{}' has been deleted.", name).green()
                );
            }
            DeleteCommands::Prompt { name } => {
                config_manager.delete_prompt(name)?;
                println!(
                    "{}",
                    format!("Prompt configuration '{}' has been deleted.", name).green()
                );
            }
        },
        Some(Commands::List { config_type }) => match config_type.as_str() {
            "models" => {
                println!("Models:");
                list_models(&config_manager, cli.verbose);
            }
            "prompts" => {
                println!("Prompts:");
                list_prompts(&config_manager);
            }
            "all" => {
                println!("Models:");
                list_models(&config_manager, cli.verbose);
                println!(
                    "
Prompts:"
                );
                list_prompts(&config_manager);
            }
            _ => {
                eprintln!(
                    "{}",
                    "Invalid config type. Use 'models', 'prompts', or 'all'.".red()
                );
            }
        },
        None => {
            // Handle chat mode
            let model = cli
                .model
                .or_else(|| config_manager.get_default_model())
                .ok_or("No model specified and no default model set")?;

            let prompt = cli.prompt.or_else(|| config_manager.get_default_prompt());

            let model_config = config_manager
                .get_model(&model)
                .ok_or(format!("Model configuration '{}' not found", model))?;

            // If input is empty,(interactive mode) wait for input, then call single_message
            if cli.input.is_empty() {
            } else {
                let input = cli.input.join(" ");
                chat::single_message(
                    &input,
                    &model_config,
                    &prompt.and_then(|p| config_manager.get_prompt(&p)).unwrap(),
                    cli.pure,
                    cli.disable_stream,
                    cli.verbose,
                )
                .await?;
            }
        }
    }

    Ok(())
}
