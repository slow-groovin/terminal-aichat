use crate::config::{ModelConfig, PromptConfig};
use async_openai::{
    Client,
    config::{Config, OpenAIConfig},
    types::{
        ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequest,
        CreateChatCompletionRequestArgs,
    },
};
use crossterm::{
    cursor::{MoveToColumn, MoveUp},
    execute,
    style::Stylize,
    terminal::{Clear, ClearType},
};
use futures::StreamExt;
use std::io::{self, Write, stdout};

pub async fn single_message(
    input: &str,
    model_config: &ModelConfig,
    prompt_config: &PromptConfig,
    pure: bool,
    disable_stream: bool,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = create_client(&model_config);

    if verbose && !pure {
        println!("Using model: {}", model_config.model_name);
    }

    if disable_stream {
        let response = client
            .chat()
            .create(create_request(input, &prompt_config, &model_config))
            .await?;

        if let Some(choice) = response.choices.first() {
            println!("\n{}", choice.message.content.as_ref().unwrap());
        }
    } else {
        let mut stream = client
            .chat()
            .create_stream(create_request(input, &prompt_config, &model_config))
            .await?;

        let mut lock = stdout().lock();
        while let Some(result) = stream.next().await {
            match result {
                Ok(response) => {
                    response.choices.iter().for_each(|chat_choice| {
                        if let Some(ref content) = chat_choice.delta.content {
                            write!(lock, "{}", content).unwrap();
                        }
                    });
                }
                Err(e) => {
                    eprintln!("\nError: {}", e);
                    break;
                }
            }
            stdout().flush()?;
        }
        println!();
    }

    Ok(())
}

fn create_client(model_config: &ModelConfig) -> Client<OpenAIConfig> {
    Client::with_config(
        OpenAIConfig::default()
            .with_api_key(&model_config.api_key)
            .with_api_base(&model_config.base_url),
    )
}

fn create_request(
    input: &str,
    prompt_config: &PromptConfig,
    model_config: &ModelConfig,
) -> CreateChatCompletionRequest {
    CreateChatCompletionRequestArgs::default()
        .model(&model_config.model_name)
        .messages([
            ChatCompletionRequestSystemMessageArgs::default()
                .content(prompt_config.content.as_ref())
                .build()
                .unwrap()
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(input)
                .build()
                .unwrap()
                .into(),
        ])
        .build()
        .unwrap()
}
