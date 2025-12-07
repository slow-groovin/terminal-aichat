<div style="text-align: center;"><img src="./aichat.webp" alt="usage" /></div>

# terminal-aichat

终端内AI/LLM聊天的CLI
- 使用Rust编写，轻量级（6.5MB二进制大小），超级快。
- 跨平台（Windows, Linux, MacOS）
- 使用 `/v1/chat/completion` API

```sh
aichat <INPUT MESSAGE>
```

## 快速入门

### 安装

```sh
# sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/slow-groovin/terminal-aichat/releases/latest/download/terminal-aichat-installer.sh | sh

# 使用cargo构建安装：
cargo install terminal-aichat

# homebrew
brew install slow-groovin/tap/terminal-aichat

# npm
npm install terminal-aichat@latest

# powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/slow-groovin/terminal-aichat/releases/latest/download/terminal-aichat-installer.ps1 | iex"
```

or 在[Release](https://github.com/slow-groovin/terminal-aichat/releases)页面中直接下载二进制程序。

### 前置要求
配置模型(以openrouter为例)
```sh
aichat set model my_model_1 --model-name openai/gpt-oss-20b:free --base-url https://openrouter.ai/api/v1 --api-key <YOUR_API_KEY>

aichat use model my_model_1
```
### chat
```sh
# 直接发送消息
aichat how to view ubuntu release version

# 如果消息与子命令冲突，用引号包裹
aichat "set swap memory to 0"

# 其他方式
aichat "<INPUT MESSAGE>"
aichat -- <INPUT MESSAGE>

# 管道
cat input.txt | aichat
cat input.txt | aichat "explain this"

# 纯净模式（不显示模型/提示配置和成本信息）
aichat --pure "Hello?"
```

## 配置和命令
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
