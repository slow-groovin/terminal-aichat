use clap::Parser;
use crate::cli::structs::{Cli, Commands, DeleteCommands, SetCommands, UseCommands};

use crate::config::*;
use crossterm::style::Stylize;

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
