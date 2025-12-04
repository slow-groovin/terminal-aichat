use super::{Config, ModelConfig};
use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor, Stylize},
};
use std::io::{self, Write};

/// 打印模型列表
pub fn print_models(config: &Config) -> io::Result<()> {
    let mut stdout = io::stdout();
    println!("{}", "Models:".on_blue().black());

    // 表头
    writeln!(
        stdout,
        "┌─────────────────────┬─────────────────────┬─────────────────────────────────────┬─────────────────┐"
    )?;
    writeln!(
        stdout,
        "│ Name                │ Model Name          │ Base URL                            │ API Key         │"
    )?;
    writeln!(
        stdout,
        "├─────────────────────┼─────────────────────┼─────────────────────────────────────┼─────────────────┤"
    )?;

    // 数据行
    for (name, model) in &config.models {
        let is_default = config.default_model.as_ref() == Some(name);
        print_model_row(&mut stdout, name, model, is_default)?;
    }

    // 底部
    writeln!(
        stdout,
        "└─────────────────────┴─────────────────────┴─────────────────────────────────────┴─────────────────┘"
    )?;

    // 显示默认模型
    if let Some(default) = &config.default_model {
        execute!(stdout, Print("Current default model: "))?;
        execute!(stdout, Print(format!("{}\n", default.clone().green())))?;
    }

    Ok(())
}

/// 打印单个模型行
fn print_model_row(stdout: &mut io::Stdout, name: &str, model: &ModelConfig, is_default: bool) -> io::Result<()> {
    let color = if is_default { Color::Green } else { Color::Reset };

    write!(stdout, "│ ")?;

    // 名称
    execute!(stdout, SetForegroundColor(color))?;
    write!(stdout, "{:<19}", truncate(name, 19))?;
    execute!(stdout, ResetColor)?;
    write!(stdout, " │ ")?;

    // 模型名
    execute!(stdout, SetForegroundColor(color))?;
    write!(
        stdout,
        "{:<19}",
        truncate(model.model_name.as_deref().unwrap_or(""), 19)
    )?;
    execute!(stdout, ResetColor)?;
    write!(stdout, " │ ")?;

    // Base URL
    execute!(stdout, SetForegroundColor(color))?;
    write!(stdout, "{:<35}", truncate(model.base_url.as_deref().unwrap_or(""), 35))?;
    execute!(stdout, ResetColor)?;
    write!(stdout, " │ ")?;

    // API Key (脱敏)
    execute!(stdout, SetForegroundColor(color))?;
    let masked = mask_api_key(model.api_key.as_deref().unwrap_or(""));
    write!(stdout, "{:<15}", masked)?;
    execute!(stdout, ResetColor)?;
    writeln!(stdout, " │")?;

    Ok(())
}

/// 打印提示列表
pub fn print_prompts(config: &Config) {
    println!("{}", "Prompts:".on_blue().black());

    for (name, prompt) in &config.prompts {
        let default_text = if config.default_prompt.as_deref() == Some(name) {
            "(default)".green()
        } else {
            "".green()
        };

        println!("{}{}: ", name.clone().blue().bold(), default_text);
        println!("{}\n{}\n{}\n", "```".blue().bold(), prompt.content, "```".blue().bold());
    }
}

/// 截断字符串
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len - 3])
    } else {
        s.to_string()
    }
}

/// 脱敏API key
fn mask_api_key(key: &str) -> String {
    if key.is_empty() {
        return String::new();
    }

    let len = key.len();
    if len <= 8 {
        "*".repeat(len)
    } else {
        format!("{}***{}", &key[..4], &key[len - 4..])
    }
}
