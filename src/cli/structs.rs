use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "aichat",
    version = "0.3.4",
    about = r#"
A terminal AI/LLM chat tool

aichat [MESSAGE]   # directly chat 
aichat [COMMAND] [ARGS]     # setting or view configs"#,
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Specify model configuration to use
    #[arg(short, long)]
    pub model: Option<String>,

    /// Specify prompt configuration to use
    #[arg(short, long)]
    pub prompt: Option<String>,

    /// Show verbose information
    #[arg(long)]
    pub verbose: bool,

    /// Use pure output mode (no extra text and color rendering)
    #[arg(long)]
    pub pure: bool,

    /// Disable streaming output
    #[arg(long)]
    pub disable_stream: bool,

    /// Specify config file path
    // #[arg(long)]
    // pub config: Option<String>,

    // #[arg(long,value_parser = non_empty_string)]
    // pub test: Option<String>,

    /// Chat input content (when no subcommand is provided)
    pub input: Vec<String>,
}

#[derive(Subcommand)]

pub enum Commands {
    /// Set model or prompt configuration
    Set {
        #[command(subcommand)]
        config: SetCommands,
    },

    /// Set default model or prompt
    Use {
        #[command(subcommand)]
        config: UseCommands,
    },

    /// Delete model or prompt configuration
    Delete {
        #[command(subcommand)]
        config: DeleteCommands,
    },

    /// List configurations
    List {
        /// Type of configuration to list (models/prompts/all)
        #[arg(default_value = "all")]
        config_type: String,
    },
}

#[derive(Subcommand)]
#[command(
    arg_required_else_help = true   // 👈 只对 Add 生效
)]
pub enum SetCommands {
    /// Set model configuration
    Model {
        /// Name of the model configuration
        #[arg(index = 1, value_parser = non_empty_string)]
        name: String,
        /// Base URL for API
        #[arg(long, value_parser = non_empty_string)]
        base_url: Option<String>,
        /// Model name
        #[arg(long, value_parser = non_empty_string)]
        model_name: Option<String>,
        /// API key
        #[arg(long)]
        api_key: Option<String>,

        #[arg(long)]
        temperature: Option<f32>,
    },
    /// Set prompt configuration
    Prompt {
        /// Name of the prompt configuration
        #[arg(index = 1, value_parser = non_empty_string)]
        name: String,
        /// Content of the prompt
        #[arg(long)]
        content: String,
    },
}

#[derive(Subcommand)]
pub enum UseCommands {
    /// Set default model
    Model {
        /// Name of the model configuration
        name: String,
    },
    /// Set default prompt
    Prompt {
        /// Name of the prompt configuration
        name: String,
    },
}

#[derive(Subcommand)]
pub enum DeleteCommands {
    /// Delete model configuration
    Model {
        /// Name of the model configuration
        name: String,
    },
    /// Delete prompt configuration
    Prompt {
        /// Name of the prompt configuration
        name: String,
    },
}

fn non_empty_string(s: &str) -> Result<String, String> {
    if s.trim().is_empty() {
        Err("param cannot be empty".to_string())
    } else {
        Ok(s.to_string())
    }
}
