use crate::cli::structs::Cli;

use super::Config;


/// 合并CLI参数和文件配置
/// 优先级: CLI > 文件配置
pub fn merge_config(file_config: &Config, cli: &Cli) -> Config {
    Config {
        models: file_config.models.clone(),
        prompts: file_config.prompts.clone(),
        
        // CLI参数优先
        default_model: cli.model.clone().or_else(|| file_config.default_model.clone()),
        default_prompt: cli.prompt.clone().or_else(|| file_config.default_prompt.clone()),
        
        // 全局标志: CLI或文件任一为true则为true
        disable_stream: cli.disable_stream || file_config.disable_stream,
        pure: cli.pure || file_config.pure,
        verbose: cli.verbose || file_config.verbose,
    }
}

