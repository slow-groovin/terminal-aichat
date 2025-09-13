use crate::cli::response_render::{RenderConfig, ResponseRenderer};
use crate::utils::StringUtils;
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
        type_speed: 30, // 50字/秒
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
            message_tx
                .send(choice.message.content.clone().unwrap_or(String::from("null")))
                .await?;
            // if let Err(err) = message_tx
            //     .send(choice.message.content.clone().unwrap_or(String::from("null")))
            //     .await
            // {
            //     eprintln!("send message failed: {}", err);
            // };
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
    renderer_handler.await?;
    log_debug!("Response Render exit.");
    if !errors.is_empty() {
        for e in errors {
            eprintln!("❌Error in sending openai-api request: {}", e);
        }
        
        return Err("failed to send request.".into());
    }
    renderer.render_tail_bar();
    Ok(())
}

fn create_client(model_config: &ModelConfig) -> Client<OpenAIConfig> {
    let env_api_key=std::env::var("OPENAI_API_KEY");
    let final_api_key=match env_api_key {
        Ok(val)=>{
            log_debug!("use env OPEN_API_KEY to override api-key.");
            val
        },
        Err(_)=>model_config.api_key.clone().unwrap_or(String::new())
    };
    log_debug!("final used api-key: {}",StringUtils::mask_sensitive(&final_api_key));
    Client::with_config(
        OpenAIConfig::default()
            .with_api_key(final_api_key)
            .with_api_base(model_config.base_url.as_ref().unwrap()),
    )
}

fn create_request(
    input: &str,
    prompt_config: &PromptConfig,
    model_config: &ModelConfig,
) -> CreateChatCompletionRequest {
    let mut builder = CreateChatCompletionRequestArgs::default();
    builder.model(model_config.model_name.as_ref().unwrap());

    match model_config.temperature {
        Some(val) => {
            builder.temperature(val);
        }
        None => {}
    };
    builder
        .temperature(model_config.temperature.unwrap())
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
