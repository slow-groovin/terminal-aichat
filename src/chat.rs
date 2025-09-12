use crate::cli::response_render::{RenderConfig, ResponseRenderer};
use crate::{
    config::{ModelConfig, PromptConfig},
    log_debug,
};
use async_openai::{
    Client,
    config::OpenAIConfig,
    error::OpenAIError,
    types::{
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequest,
        CreateChatCompletionRequestArgs,
    },
};
use futures::StreamExt;

pub async fn completion(
    input: &str,
    model_config_name: String,
    model_config: &ModelConfig,
    prompt_config_name: String,
    prompt_config: &PromptConfig,
    pure: bool,
    disable_stream: bool,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let _ = verbose;
    let client = create_client(&model_config);
    let model_name = model_config.model_name.as_ref().unwrap();
    // 创建渲染器配置
    let config = RenderConfig {
        pure,
        model_config_name: model_config_name,
        model_name: model_name.to_string(),
        prompt_config_name,
        type_speed: 50, // 50字/秒
        disable_stream: disable_stream,
    };
    let mut errors = Vec::<OpenAIError>::new();
    let renderer = ResponseRenderer::new();
    let (message_tx, renderer_handler) = renderer.start_render(config);

    if disable_stream {
        log_debug!("Start send chat request.");
        let response = client
            .chat()
            .create(create_request(input, &prompt_config, &model_config))
            .await?;
        log_debug!("Received chat response.");

        if let Some(choice) = response.choices.first() {
            if let Err(err) = message_tx
                .send(choice.message.content.clone().unwrap_or(String::from("null")))
                .await
            {
                eprintln!("send message failed: {}", err);
            };
        }
    } else {
        let mut stream = client
            .chat()
            .create_stream(create_request(input, &prompt_config, &model_config))
            .await?;

        log_debug!("Start receive stream message.");
        while let Some(result) = stream.next().await {
            match result {
                Ok(response) => {
                    for chat_choice in response.choices.iter() {
                        if let Some(content) = &chat_choice.delta.content {
                            if let Err(_) = message_tx.send(content.clone()).await {};
                        }
                    }
                }
                Err(e) => {
                    errors.push(e);
                }
            }
        }
        log_debug!("Exit receive stream message.");
    }
    drop(message_tx);
    log_debug!("Drop Message Sender.");
    let _ = renderer_handler.await;
    log_debug!("Response Render exit.");
    if !errors.is_empty() {
        for e in errors {
            println!("Error happends: {}", e);
        }
    }
    Ok(())
}

fn create_client(model_config: &ModelConfig) -> Client<OpenAIConfig> {
    Client::with_config(
        OpenAIConfig::default()
            .with_api_key(&model_config.api_key.clone().unwrap_or(String::new()))
            .with_api_base(model_config.base_url.as_ref().unwrap()),
    )
}

fn create_request(
    input: &str,
    prompt_config: &PromptConfig,
    model_config: &ModelConfig,
) -> CreateChatCompletionRequest {
    CreateChatCompletionRequestArgs::default()
        .model(model_config.model_name.as_ref().unwrap())
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
