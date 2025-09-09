use crate::config::{ModelConfig, PromptConfig};
use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequest,
        CreateChatCompletionRequestArgs,
    },
};
use crate::cli::response_render::{ResponseRenderer, RenderConfig, ResponseStatus};
use futures::StreamExt;

pub async fn completion(
    input: &str,
    model_config: &ModelConfig,
    prompt_config: &PromptConfig,
    pure: bool,
    disable_stream: bool,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = create_client(&model_config);
    let model_name = model_config.model_name.as_ref().unwrap();

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

        // 创建渲染器配置
        let mut config = RenderConfig {
            pure,
            model_name: model_name.to_string(),
            prompt_name: "default".to_string(),
            type_speed: 50, // 50字/秒
            status_refresh_interval: 1000, // 1秒刷新一次状态
        };
        let renderer = ResponseRenderer::new(config);

        while let Some(result) = stream.next().await {
            match result {
                Ok(response) => {
                    for chat_choice in response.choices.iter() {
                        if let Some(ref content) = chat_choice.delta.content {
                            renderer.push_content(content).await?;
                        }
                    }
                }
                Err(e) => {
                    renderer.set_status(ResponseStatus::Error).await?;
                    eprintln!("\nError: {}", e);
                    break;
                }
            }
        }

        renderer.set_status(ResponseStatus::Done).await?;
        let mut renderer=renderer;
        renderer.wait().await;
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
