## 综述
这是一个简单的命令行工具， 用于在终端进行快速的llm chat：
- 跨平台， 基于rust开发
- 使用openai chat 接口进行调用

## 使用框架
- clap  (cli)
- async_openai  (ai sdk)
- crossterm  (terminal output render)

## config
- 配置model： 
    - 支持添加多个配置
    - 每个配置包含：name(as configName or configId)， baseURL, modelName, apiKey
    - 可以制定默认配置
- 配置sys prompt:
    - 支持添加多个配置
    - 每个配置包含：name(as configName or configId)，， content
    - 可以制定默认配置
    - 默认包含一个预置的项


## command
### chat
```sh
aichat <input content>
aichat hello? who are you?
aichat how to view release information
aichat -m qwen3 -p bash-prompt2  how to view release information

# interactive mode
aichat
<input content and press Enter>
```

#### 参数
- -m --model: 通过model name指定模型配置 
- -p --prompt: 通过prompt name制定提示词配置
- `--verbose`
- `--pure`: 使用纯净模式, 输出响应content, 不使用crossterm渲染耗时, usage, 当前使用的model,prompt等信息
- `--disable-stream`: 禁用流式输出
- `--config <filepath> `


### model/prompt configs

#### create or update model config
```sh
aichat set model <modelConfigName> --baseURL <baseUrl> --modelName <modelName> --apiKey <apiKey>
##example
aichat set model gpt-account1  --baseURL https://openrouter.ai/api/v1 --modelName openai/gpt-5 --apiKey sk-ae7721eb147977aed7779f1
aichat set model qwen-local --baseURL https://localhost:3000/v1 --modelName qwen3-8b
```
#### create or update prompt config
```sh
aichat set prompt <promptConfigName> --content <content>
##example
aichat set prompt my-prompt --content "You are a helpful assistant that give shell command."
```

#### set default config
```sh
aichat use model <modelConfigName>
aichat use prompt <promptConfigName>
```

#### delete config
```sh
aichat delete model <modelConfigName>
aichat delete prompt <promptConfigName>
```


#### list configs
```sh
aichat list models
aichat list prompts
aichat list 
```

## config file

- default location: `~/.terminal-aichat/config.json`
- randome key localtion (for enc/dec apiKeys): `~/.terminal-aichat/aes_key.bin`


```json
{
    "models":{
        "gpt-account1":{
            "modelName":"qwen3",
            "baseURL":"https://openrouter.ai/api/v1",
            "apiKey":"<encryption>",
        }
    },
    "prompts":{
        "bash-prompt2":{
            "content":"You are a helpful assistant that give shell command."
        },
    },
    "default-model":"gpt-account1",
    "default-prompt":"bash-prompt2",
    "disable-stream":true,
    "pure":true
}
```
## environment variables
### force override model config
```sh
export OPENAI_API_KEY=
export OPENAI_BASE_URL=
export OPENAI_MODEL_NAME=
```

## output样式设计
用户输入（不清空）
第一行: 状态栏: `<status:running(...动画)|done|error (对应的颜色作为背景色)>   model:<modelConfigName> prompt:<promptConfigName>  `
第二行：流式输出： .......

如果`--pure`则仅流式输出