# terminal-aichat
[README中文](./README_zh.md)

A terminal AI/LLM chat CLI
- very simple, super fast, lightweight, and cross platform(Windows, Linux, MacOs)
- using `/v1/chat/completion` API

```sh
aichat <INPUT MESSAGE>    # chat 
aichat "<INPUT MESSAGE>"  # chat
aichat -- <INPUT MESSAGE> # chat
cat input.txt | aichat   # chat
cat input.txt | aichat "explain this"   # chat
```

## Quick Start

### Installation

```sh
cargo install terminal-aichat
```

or

Download binary from Release

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
```

## Usage Examples

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
