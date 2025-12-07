<div align="center">
<h1>terminal-aichat</h1>
</div>


<div align="center"><img src="./aichat.webp" alt="terminal-aichat" height="140" /></div>

[README中文](./README_zh.md)

A CLI for AI/LLM chat in terminal
- written in rust, light (6.5MB binary size), super fast.
- multi platform(Windows, Linux, MacOS)
- using `/v1/chat/completion` API



```sh
aichat <INPUT MESSAGE>    
```



## Quick Start

### Installation


#### sh
```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/slow-groovin/terminal-aichat/releases/latest/download/terminal-aichat-installer.sh | sh
```

#### cargo
```sh
cargo install terminal-aichat
```

#### homebrew
```sh
brew install slow-groovin/tap/terminal-aichat
```

#### npm
```sh
npm install terminal-aichat@latest
```

#### powershell
```sh
powershell -ExecutionPolicy Bypass -c "irm https://github.com/slow-groovin/terminal-aichat/releases/latest/download/terminal-aichat-installer.ps1 | iex"
```

#### binary
or download executable binaries directly in [Release](https://github.com/slow-groovin/terminal-aichat/releases) page.

### Prerequisites

Configure a model (example with OpenRouter):

```sh
aichat set model my_model_1 --model-name openai/gpt-oss-20b:free --base-url https://openrouter.ai/api/v1 --api-key <YOUR_API_KEY>

aichat use model my_model_1
```

### Chat

```sh
# Directly send a message
aichat how to view ubuntu release version

# If your message conflicts with a subcommand, wrap it with quotes
aichat "set swap memory to 0"

# other ways
aichat "<INPUT MESSAGE>"  
aichat -- <INPUT MESSAGE> 

# pipe
cat input.txt | aichat   
cat input.txt | aichat "explain this"

# pure mode (display for model/prompts configs and costs will be hide)
aichat --pure "Hello?"
```

## Configurations And Commands

### View Configurations

```sh
aichat list
aichat list model
aichat list prompt
```

### Configure Prompts

```sh
aichat set prompt <PROMPT_CONFIG_NAME> --content "your prompt content"
aichat set prompt my_prompt_1 --content "use plain text, give extremely concise output"
```

### Update Model Configuration (Partial Update)

```sh
aichat set model my_model_1 --temperature 0.3 --model-name gpt-4o
```

### Set Model Temperature

```sh
aichat set model my_model_1 --temperature 0.3
```

### Delete a Configuration Item

```sh
aichat delete model sample_model_gpt
```

### Use Temporary API Key via Environment Variable

> Useful for avoiding persistent API key storage or for testing.
> it will override API key in final request.

```sh
export OPENAI_API_KEY=sk-***************
aichat "Hello?"
```

### Configuration Files

> On first run, the config file is automatically initialized.

* `~/.terminal-aichat/config.json` — stores configuration JSON
* `~/.terminal-aichat/aes_key.bin` — stores the random encryption key for securing API keys (to avoid plaintext storage)

```sh
cat ~/.terminal-aichat/config.json
```

### Set Log Level

```sh
export LOG_LEVEL=DEBUG
```

> Equivalent to using `--verbose`

### Pure Mode (`--pure`)

> Suppresses all extra messages and outputs only the response.

```sh
aichat --pure "Hello?"
```

### Verbose Logging (`--verbose`)

```sh
aichat --verbose "Hello?"
```

### Disable Streaming Mode (`--disable-stream`)

```sh
aichat --disable-stream "Hello?"
```
