use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "termchat")]
#[command(about = "A terminal chat tool for LLM", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Chat input content (when no subcommand is provided)
    pub input: Vec<String>,

    /// Specify model configuration to use
    #[arg(short, long)]
    pub model: Option<String>,

    /// Specify prompt configuration to use
    #[arg(short, long)]
    pub prompt: Option<String>,

    /// Show verbose information
    #[arg(long)]
    pub verbose: bool,

    /// Use pure output mode (no rendering)
    #[arg(long)]
    pub pure: bool,

    /// Disable streaming output
    #[arg(long)]
    pub disable_stream: bool,

    /// Specify config file path
    #[arg(long)]
    pub config: Option<String>,
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
pub enum SetCommands {
    /// Set model configuration
    Model {
        /// Name of the model configuration
        #[arg(index = 1)]
        name: String,
        /// Base URL for API
        #[arg(long)]
        base_url: String,
        /// Model name
        #[arg(long)]
        model_name: String,
        /// API key
        #[arg(long)]
        api_key: Option<String>,
    },
    /// Set prompt configuration
    Prompt {
        /// Name of the prompt configuration
        #[arg(index = 1)]
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
