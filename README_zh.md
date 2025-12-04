# terminal-aichat
简单易用, 超级快, 跨平台(Windows, Linux, MacOs)的 AI CHAT CLI, 使用 Openai-compatible `/v1/chat/completion` API 在终端内进行快捷chat

```sh
aichat <INPUT MESSAGE>    # chat 
aichat "<INPUT MESSAGE>"  # chat
aichat -- <INPUT MESSAGE> # chat
cat input.txt | aichat   # chat
cat input.txt | aichat "explain this"   # chat
```

## 快速入门
### 安装
```sh
cargo install terminal-aichat
```

or

从 Release 中下载二进制程序

### 前置要求
配置模型(以openrouter为例)
```sh
aichat set model my_model_1 --model-name openai/gpt-oss-20b:free --base-url https://openrouter.ai/api/v1 --api-key <YOUR_API_KEY>

aichat use model my_model_1
```
### chat
```sh
# Directly send a message
aichat how to view ubuntu release version

# If your message conflicts with a subcommand, wrap it with quotes
aichat "set swap memory to 0"

```

### 使用示例
#### 查看配置
```sh
aichat list
aichat list model
aichat list prompt
```
#### 配置prompt

```sh
aichat set prompt <PROMPT_CONFIG_NAME> --content "your prompt content"
aichat set prompt my_prompt_1 --content "use plain text, give extremly concise output"
```
#### 部分更新model配置

```sh
aichat set model my_model_1 --temperature 0.3 --model-name gpt-4o
```
#### 设置model temperature
```sh
aichat set model my_model_1 --temperature 0.3 
```
#### 删除配置项
```sh
aichat delete model sample_model_gpt
```

#### 使用临时环境变量指定 api-key
> 如果是需要避免将api-key持久化存储, 或者测试用途, 可以使用`OPENAI_API_KEY`强制覆盖最终发送请求的api-key
```sh
export OPENAI_API_KEY=sk-***************
aichat "Hello?"
```
#### 配置文件
> 第一次运行程序时, 会自动初始化配置文件

- `~/.terminal-aichat/config.json` 存储配置json
- `~/.terminal-aichat/aes_key.bin` 存储api-key的随机加密key(api-key的加密的目的是避免明文方式本地存储)

```sh
cat ~/.terminal-aichat/config.json
```
#### 设置日志级别
```sh
export LOG_LEVEL=DEBUG
```
> 目前, 等同于 `--verbose`

#### 使用纯净模式 (`--pure`)
> 纯净模式不显示任何提示信息

```sh
aichat --pure "Hello?"
```
#### 显示详细日志 (`--verbose`)
```sh
aichat --verbose "Hello?"
```
#### 不使用stream方式调用api (`--disable-stream`)
```sh
aichat --disable-stream "Hello?"
```






