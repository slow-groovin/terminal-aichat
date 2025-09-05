use async_openai::Client;
use async_openai::types::ChatCompletionRequestUserMessageArgs;
use async_openai::{
    config::OpenAIConfig,
    types::{ChatCompletionRequestAssistantMessageArgs, CreateChatCompletionRequestArgs},
};
use futures::StreamExt;
use std::error::Error;
use std::io::{Write, stdout};
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _ = dotenvy::dotenv();
    // Create client
    let client = Client::with_config(
        OpenAIConfig::default().with_api_base(std::env::var("OPENAI_BASEURL").unwrap()),
    );

    // Create request using builder pattern
    // Every request struct has companion builder struct with same name + Args suffix
    let request = CreateChatCompletionRequestArgs::default()
        .model("qwen-flash")
        .messages([
            ChatCompletionRequestUserMessageArgs::default()
                .content("hello")
                .build()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content("你是谁？")
                .build()?
                .into(),
            ChatCompletionRequestAssistantMessageArgs::default()
                .content("好呀~我是gpt-2.5-opus，是openai研发的超大规模语言模型。你可以叫我gpt。我是一个能够回答问题、创作文字、编程、表达观点的AI GPT。很高兴认识你！有什么我可以帮你的吗？😊")
                .build()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content("我刚才没听清， 你的名字叫什么？")
                .build()?
                .into(),
        ])
        .build()?;

    println!("{}", serde_json::to_string(&request).unwrap());
    let mut stream = client.chat().create_stream(request).await?;

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
            Err(err) => {
                writeln!(lock, "error: {err}").unwrap();
            }
        }
        stdout().flush()?;
    }

    Ok(())
}
