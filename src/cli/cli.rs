use std::process::exit;

use crate::cli::interactive::interactive_input;
use crate::cli::structs::{Cli, Commands, DeleteCommands, SetCommands, UseCommands};
use crate::config::{ConfigManager, ModelConfig, PromptConfig};
use crate::utils::StringUtilsTrait;
use crate::utils::logger::set_log_level;
use crate::{chat, log_debug, utils};
use clap::Parser;
use crossterm::style::Stylize;
use utils::logger::{self};

pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // if !cli.test.is_none() {
    //     // test();
    //     // return Ok(());
    // }
    // Initialize config manager
    let mut config_manager = ConfigManager::load()?;

    //if config file not exist, try to init and save;
    config_manager.try_initialize_default_configs_and_save()?;

    
    if config_manager.config.verbose.unwrap() {
        set_log_level(logger::LogLevel::Trace);
    }
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
            handle_list_command(&config_manager, config_type).await?;
        }
        None => {
            log_debug!("match None Command");
            handle_chat_command(&cli, &config_manager).await?;
        }
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
            temperature,
        } => {
            config_manager.set_model(
                name.clone(),
                ModelConfig {
                    base_url: base_url.clone(),
                    model_name: model_name.clone(),
                    api_key: api_key.clone(),
                    temperature: temperature.clone()
                },
            )?;
            println!("{}", format!("Model configuration '{}' has been set.", name).green());
        }
        SetCommands::Prompt { name, content } => {
            config_manager.set_prompt(
                name.clone(),
                PromptConfig {
                    content: content.clone(),
                },
            )?;
            println!("{}", format!("Prompt configuration '{}' has been set.", name).green());
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
                println!("{}", format!("Default model has been set to '{}'.", name).green());
            } else {
                eprintln!("{}", format!("Model configuration '{}' not found.", name).red());
            }
        }
        UseCommands::Prompt { name } => {
            if config_manager.get_prompt(name).is_some() {
                config_manager.set_default_prompt(name.clone())?;
                println!("{}", format!("Default prompt has been set to '{}'.", name).green());
            } else {
                eprintln!("{}", format!("Prompt configuration '{}' not found.", name).red());
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
) -> Result<(), Box<dyn std::error::Error>> {
    if config_type == "models" || config_type == "model" || config_type == "all" {
        println!("{}", "Models:".on_blue().black());
        config_manager.list_models();
    }

    println!("\n");

    if config_type == "prompts" || config_type == "prompt" || config_type == "all" {
        println!("{}", "Prompts:".on_blue().black());
        config_manager.list_prompts();
    }

    println!("config file location: {}", "~/.terminal-aichat/config.json".cyan());

    Ok(())
}

async fn handle_chat_command(cli: &Cli, config_manager: &ConfigManager) -> Result<(), Box<dyn std::error::Error>> {
    let model_name = config_manager.get_default_model_name();
    let prompt_name = config_manager.get_default_prompt_name();
    let model_hint = format!(
        "{} to list,\n{} to set, \n{} to specify default, \n{} to temporarily specify.",
        "aichat config list model".dark_green(),
        "aichat set model <MODEL_CONFIG_NAME> --model-name <MODEL_NAME> --base-url <BASE_URL> --api-key <API_KEY>"
            .dark_green(),
        "aichat use model <MODEL_CONFIG_NAME>".dark_green(),
        "-m <MODEL_CONFIG_NAME>".dark_green()
    );
    let prompt_hint = format!(
        "{} to list,\n{} to set, \n{} to specify default, \n{} to temporarily specify.",
        "aichat config list prompt".dark_green(),
        "aichat set prompt <PROMPT_CONFIG_NAME> --content <PROMPT_CONTENT>".dark_green(),
        "aichat use prompt <PROMPT_CONFIG_NAME>".dark_green(),
        "-p <PROMPT_CONFIG_NAME>".dark_green()
    );
    if matches!(model_name.as_deref(), None | Some("")) {
        eprintln!("❌ No model config specified, please:\n{}", model_hint);
        exit(78);
    }
    if matches!(prompt_name.as_deref(), None | Some("")) {
        eprintln!("❌ No prompt config specified, please:\n{}", prompt_hint);
        exit(78);
    }

    let model_name: &str = model_name.as_ref().unwrap();
    let prompt_name: &str = prompt_name.as_ref().unwrap();

    let model_config = config_manager.get_model(model_name).unwrap_or_else(|| {
        eprintln!(
            "❌Model configuration '{}' not found, please:\n{}",
            model_name.blue(),
            model_hint
        );
        std::process::exit(78);
    });

    let prompt_config = config_manager.get_prompt(prompt_name).unwrap_or_else(|| {
        eprintln!(
            "❌Prompt configuration '{}' not found, please:\n{}",
            prompt_name.blue(),
            prompt_hint
        );
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

    if input.trim().is_empty() {
        println!("{}", "⚠ Input message is empty.".yellow());
        exit(1);
    }

    log_debug!(
        "Begin to chat. model: {}, prompt: {}, input: {}...",
        model_name,
        prompt_name,
        &input.safe_substring(20)
    );
    chat::completion(
        &input,
        model_name.to_string(),
        &model_config,
        prompt_name.to_string(),
        &prompt_config,
        config_manager.config.pure.unwrap(),
        config_manager.config.disable_stream.unwrap(),
        config_manager.config.verbose.unwrap(),
    )
    .await?;

    log_debug!("Chat Done.");
    Ok(())
}
