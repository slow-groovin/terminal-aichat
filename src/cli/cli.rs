use std::io::{self, IsTerminal, Read};
use std::process::exit;

use crate::cli::interactive::interactive_input;
use crate::cli::structs::{Cli, Commands, DeleteCommands, SetCommands, UseCommands};

use crate::config::{
    Config, ConfigBuilder, ConfigManager, ModelConfig, PromptConfig, merge_config, print_models, print_prompts,
};
use crate::utils::StringUtilsTrait;
use crate::utils::logger::set_log_level;
use crate::{chat, log_debug, utils};
use clap::Parser;
use crossterm::style::Stylize;
use utils::logger::{self};

pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut custom_args = std::env::args().collect::<Vec<_>>();

    if !io::stdin().is_terminal() {
        //if has pipe stdin
        let mut input = String::new();
        io::stdin().read_to_string(&mut input).unwrap_or_default();
        custom_args.push(input.trim().to_string());
    }

    let cli = Cli::parse_from(custom_args);

    let config_dir = ConfigManager::get_config_dir()?;
    let mut config_manager = ConfigManager::new(&config_dir)?;
    let mut file_config = config_manager.load()?;

    // 如果配置文件不存在,初始化默认配置
    if !config_manager.exists() {
        file_config = ConfigBuilder::new(file_config).with_defaults().build();
        config_manager.save(&file_config)?;
    }

    let runtime_config = merge_config(&file_config, &cli);

    if runtime_config.verbose {
        set_log_level(logger::LogLevel::Trace);
    }

    // Handle subcommands
    match &cli.command {
        Some(Commands::Set { config }) => {
            handle_set_command(&mut file_config, &mut config_manager, config).await?;
        }
        Some(Commands::Use { config }) => {
            handle_use_command(&mut file_config, &mut config_manager, config).await?;
        }
        Some(Commands::Delete { config }) => {
            handle_delete_command(&mut file_config, &mut config_manager, config).await?;
        }
        Some(Commands::List { config_type }) => {
            handle_list_command(&mut file_config, config_type).await?;
        }
        None => {
            log_debug!("match None Command");
            handle_chat_command(&runtime_config, &mut &cli).await?;
        }
    }

    Ok(())
}

async fn handle_set_command(
    file_config: &mut Config,
    config_manager: &mut ConfigManager,
    set_command: &SetCommands,
) -> io::Result<()> {
    match set_command {
        SetCommands::Model {
            name,
            base_url,
            model_name,
            api_key,
            temperature,
        } => {
            let mut new_model = ModelConfig {
                base_url: base_url.clone(),
                model_name: model_name.clone(),
                api_key: api_key.clone(),
                temperature: temperature.clone(),
            };
            let raw_model = file_config.models.get(name).clone();
            match raw_model {
                Some(raw_model) => {
                    //merged
                    new_model = new_model.merge_with(raw_model);
                }
                None => {}
            };
            file_config.models.insert(name.clone(), new_model);
            config_manager.save(file_config)?;
            // config_manager.save(
            //     name.clone(),
            //     ModelConfig?;
            println!("{}", format!("Model configuration '{}' has been set.", name).green());
        }
        SetCommands::Prompt { name, content } => {
            file_config.prompts.insert(
                name.clone(),
                PromptConfig {
                    content: content.clone(),
                },
            );
            config_manager.save(file_config)?;
            println!("{}", format!("Prompt configuration '{}' has been set.", name).green());
        }
    }
    Ok(())
}

async fn handle_use_command(
    file_config: &mut Config,
    config_manager: &mut ConfigManager,
    use_command: &UseCommands,
) -> Result<(), Box<dyn std::error::Error>> {
    match use_command {
        UseCommands::Model { name } => {
            if file_config.models.get(name).is_some() {
                file_config.default_model = Some(name.clone());
                config_manager.save(file_config)?;
                println!("{}", format!("Default model has been set to '{}'.", name).green());
            } else {
                eprintln!("{}", format!("Model configuration '{}' not found.", name).red());
            }
        }
        UseCommands::Prompt { name } => {
            if file_config.prompts.get(name).is_some() {
                file_config.default_prompt = Some(name.clone());
                config_manager.save(file_config)?;
                println!("{}", format!("Default prompt has been set to '{}'.", name).green());
            } else {
                eprintln!("{}", format!("Prompt configuration '{}' not found.", name).red());
            }
        }
    }
    Ok(())
}

async fn handle_delete_command(
    file_config: &mut Config,
    config_manager: &mut ConfigManager,
    delete_command: &DeleteCommands,
) -> Result<(), Box<dyn std::error::Error>> {
    match delete_command {
        DeleteCommands::Model { name } => {
            file_config.models.remove(name);
            config_manager.save(file_config)?;
            println!(
                "{}",
                format!("Model configuration '{}' has been deleted.", name).green()
            );
        }
        DeleteCommands::Prompt { name } => {
            file_config.prompts.remove(name);
            config_manager.save(file_config)?;
            println!(
                "{}",
                format!("Prompt configuration '{}' has been deleted.", name).green()
            );
        }
    }
    Ok(())
}

async fn handle_list_command(file_config: &mut Config, config_type: &String) -> Result<(), Box<dyn std::error::Error>> {
    if config_type == "models" || config_type == "model" || config_type == "all" {
        print_models(file_config)?;
    }

    println!("\n");

    if config_type == "prompts" || config_type == "prompt" || config_type == "all" {
        print_prompts(file_config);
    }

    let config_path = ConfigManager::get_config_dir()?.join("config.json");
    println!("config file location: {}", config_path.display().to_string().cyan());

    Ok(())
}

async fn handle_chat_command(runtime_config: &Config, cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    let model_name = runtime_config.default_model.clone();
    let prompt_name = runtime_config.default_prompt.clone();
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

    let model_config = runtime_config.models.get(model_name).unwrap_or_else(|| {
        eprintln!(
            "❌Model configuration '{}' not found, please:\n{}",
            model_name.blue(),
            model_hint
        );
        std::process::exit(78);
    });

    let prompt_config = runtime_config.prompts.get(prompt_name).unwrap_or_else(|| {
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
        runtime_config.pure,
        runtime_config.disable_stream,
        runtime_config.verbose,
    )
    .await?;

    log_debug!("Chat Done.");
    Ok(())
}
