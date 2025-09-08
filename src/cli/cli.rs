use std::process::exit;

use crate::{chat, log_debug, utils};
use crate::cli::interactive::interactive_input;
use crate::cli::structs::{Cli, Commands, DeleteCommands, SetCommands, UseCommands};
use clap::Parser;
use utils::logger::{self};
use crate::config::{ConfigManager, ModelConfig, PromptConfig};
use crossterm::style::Stylize;

pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Initialize config manager
    let mut config_manager = ConfigManager::new()?;

    // Handle subcommands
    match &cli.command {
        Some(Commands::Set { config }) => {
            handle_set_command(&mut config_manager, config).await?;
        }
        Some(Commands::Use { config }) => {
            handle_use_command(&mut config_manager, config).await?;
        }
        Some(Commands::Delete { config }) => {
            handle_delete_command(&mut config_manager, config).await?;
        }
        Some(Commands::List { config_type }) => {
            handle_list_command(&config_manager, config_type, cli.verbose).await?;
        }
        Some(Commands::Init {}) => {
            handle_init().await?;
        }
        None => {
            handle_chat_command(&cli, &config_manager).await?;
        }
    }
    Ok(())
}

async fn handle_init() -> Result<(), Box<dyn std::error::Error>> {
    let mut config_manager = ConfigManager::new()?;

    if config_manager.config_file_exists() {
        println!(
            "{}",
            "Configuration file already exists. No new configuration created.".yellow()
        );
    } else {
        let (model_name, prompt_name) = config_manager.initialize_default_configs()?;
        println!(
            "{}",
            format!(
                "Default configurations for model '{}' and prompt '{}' have been initialized.",
                model_name, prompt_name
            )
            .green()
        );
    }
    Ok(())
}

async fn handle_set_command(
    config_manager: &mut ConfigManager,
    set_command: &SetCommands,
) -> Result<(), Box<dyn std::error::Error>> {
    match set_command {
        SetCommands::Model {
            name,
            base_url,
            model_name,
            api_key,
        } => {
            let final_api_key = api_key
                .clone()
                .or_else(|| std::env::var("OPENAI_API_KEY").ok());

            config_manager.set_model(
                name.clone(),
                ModelConfig {
                    base_url: base_url.clone(),
                    model_name: model_name.clone(),
                    api_key: final_api_key,
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
    }
    Ok(())
}

async fn handle_use_command(
    config_manager: &mut ConfigManager,
    use_command: &UseCommands,
) -> Result<(), Box<dyn std::error::Error>> {
    match use_command {
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
    }
    Ok(())
}

async fn handle_delete_command(
    config_manager: &mut ConfigManager,
    delete_command: &DeleteCommands,
) -> Result<(), Box<dyn std::error::Error>> {
    match delete_command {
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
    }
    Ok(())
}

async fn handle_list_command(
    config_manager: &ConfigManager,
    config_type: &String,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    match config_type.as_str() {
        "models" => {
            println!("Models:");
            list_models(config_manager, verbose);
        }
        "prompts" => {
            println!("Prompts:");
            list_prompts(config_manager);
        }
        "all" => {
            println!("Models:");
            list_models(config_manager, verbose);
            println!(
                "
Prompts:"
            );
            list_prompts(config_manager);
        }
        _ => {
            eprintln!(
                "{}",
                "Invalid config type. Use 'models', 'prompts', or 'all'.".red()
            );
        }
    }
    Ok(())
}

async fn handle_chat_command(
    cli: &Cli,
    config_manager: &ConfigManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let model_name = cli.model.clone().or(config_manager.get_default_model());

    let prompt_name = cli
        .prompt
        .clone()
        .or_else(|| config_manager.get_default_prompt());

    if matches!(model_name.as_deref(), None | Some("")) {
        println!(
            "❌ No model config specified, please use {} to set or  {} to specify.",
            "aichat use model <MODEL_CONFIG_NAME>".green(),
            "-m <MODEL_CONFIG_NAME>".green()
        );
        exit(78);
    }
    if matches!(prompt_name.as_deref(), None | Some("")) {
        println!(
            "❌ No prompt config specified, please use {} to set or  {} to specify.",
            "aichat use prompt <PROMPT_CONFIG_NAME>".green(),
            "-p <PROMPT_CONFIG_NAME>".green()
        );
        exit(78);
    }

    let model_name:&str=model_name.as_ref().unwrap();
    let prompt_name:&str=prompt_name.as_ref().unwrap();

    let model_config = config_manager.get_model(model_name).unwrap_or_else(|| {
            println!("❌Model configuration '{}' not found", model_name.blue());
            std::process::exit(78);
        });

    let prompt_config = config_manager.get_prompt(prompt_name).unwrap_or_else(|| {
        println!("❌Prompt configuration '{}' not found", prompt_name.blue());
        std::process::exit(78);
    });

    // If input is empty,(interactive mode) wait for input, then call single_message
    let input = if cli.input.is_empty() {
        let input = interactive_input().await?;
        input
    } else {
        let input = cli.input.join(" ");
        input
    };

    if input.trim().is_empty(){
        println!("{}","⚠ Input message is empty.".yellow());
        exit(1);
    }


    log_debug!("Begin to chat. model: {}, prompt: {}, input: {}...",model_name,prompt_name, &input[0..20]);
    chat::completion(
        &input,
        &model_config,
        &prompt_config,
        cli.pure,
        cli.disable_stream,
        cli.verbose,
    )
    .await?;

    log_debug!("Chat Done.");
    Ok(())
}

pub fn list_models(config_manager: &ConfigManager, verbose: bool) {
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
                    config.model_name.unwrap_or(String::from("null")),
                    config.base_url.unwrap_or(String::from("null"))
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

pub fn list_prompts(config_manager: &ConfigManager) {
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
