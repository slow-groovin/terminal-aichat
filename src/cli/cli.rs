use clap::Parser;
use crate::chat;
use crate::cli::config::{list_models,list_prompts};
use crate::cli::structs::{Cli, Commands, DeleteCommands, SetCommands, UseCommands};

use crate::config::{ConfigManager,ModelConfig, PromptConfig,};
use crossterm::style::Stylize;


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
    let model = cli
        .model
        .clone()
        .or_else(|| config_manager.get_default_model())
        .ok_or("No model specified and no default model set")?;

    let prompt_name = cli.prompt.clone().or_else(|| config_manager.get_default_prompt());

    let model_config = config_manager
        .get_model(&model)
        .ok_or(format!("Model configuration '{}' not found", model))?;

    // If input is empty,(interactive mode) wait for input, then call single_message
    if cli.input.is_empty() {
        // interactive_input().await?; // This function is not implemented yet
        eprintln!("{}", "Interactive mode not yet implemented.".red());
    } else {
        let input = cli.input.join(" ");
        let prompt_config = if let Some(p_name) = prompt_name {
            config_manager.get_prompt(&p_name).ok_or(format!("Prompt configuration '{}' not found", p_name))?
        } else {
            return Err("No prompt specified and no default prompt set".into());
        };

        chat::completion(
            &input,
            &model_config,
            &prompt_config,
            cli.pure,
            cli.disable_stream,
            cli.verbose,
        )
        .await?;
    }
    Ok(())
}

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
        None => {
            handle_chat_command(&cli, &config_manager).await?;
        }
    }
    println!("DEBUG: DONE.");
    Ok(())
}

/**
 * 接受用户输入, shift+enter换行, enter确定. 返回input内容
 */
async fn interactive_input(){

}
