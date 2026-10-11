# OpenAI-compatible API configuration and binary discovery

## Fixed

The local Neovim config pointed to `core/target/release/hexwitch` without a release
build. Build the engine at that path; the wrapper now reports candidate paths and
can discover PATH, then checkout release/debug binaries when cmd is omitted.
Explicit cmd is respected; no surprising download/build or fallback to another
binary when an explicit path is wrong. `checkhealth` shows the resolved path.

## One API transport

All live requests use OpenAI-compatible Chat Completions. Configure the API root,
model ID and key environment-variable name instead of provider-specific SDKs:

```lua
require("hexwitch").setup({
  base_url = vim.env.OPENAI_BASE_URL or "https://api.openai.com/v1",
  api_key_env = "OPENAI_API_KEY",
  model = vim.env.HEXWITCH_MODEL or "gpt-4o-mini",
  response_format = "json_schema",
  auto_apply = false,
})
```

The engine appends `/chat/completions`. `--base-url`, `--api-key-env`, `--model`
and `--response-format` expose the same CLI connection. OPENAI_BASE_URL is the
standard environment override; the named key environment is read only on live
calls. Defaults use the OpenAI root and OPENAI_API_KEY. Legacy --provider aliases
(openai/openrouter) remain convenience defaults; no new provider enum is needed.

json_schema requests server-side strict structured output; json_object uses JSON
mode; none omits response_format. Non-schema modes include the exact schema in
the system prompt and still run strict local validation. No automatic retry or
format downgrade silently incurs more requests. Sampling temperature is omitted
rather than forcing an unsupported parameter on reasoning-model APIs.

Both HTTP and HTTPS are accepted, including explicit remote/internal HTTP gateways.
HTTPS remains recommended: HTTP sends credentials/prompts unencrypted and cannot
protect against interception or modification. CLI and health checks warn, without
printing the credential or payload. Parsed URLs reject credentials, query and fragment,
so keys cannot accidentally enter argv/logs via URL configuration. Credentials
and payload travel over stdin, never curl argv or temporary files. Redirects remain
disabled; redacted provider failures, response limits and completion/refusal checks
remain. Health checks inspect key presence only and hide malformed secret URLs.

## Validation

Real loopback TCP mocks drive the CLI through custom API path, model and key name
in all three output modes. Environment base URL is tested. Negative endpoint
fixtures reject unsupported protocols, URL credentials and accidental full endpoint
paths; explicit remote HTTP roots are accepted with warnings. Headless Neovim tests check discovery, explicit-path refusal and argument
forwarding. No real provider request, key capture or personal data transmission was
performed during this package.

## Limitations

OpenAI-compatible does not guarantee every provider/model accepts strict schemas,
role names, or response shapes. Select output mode explicitly. No Anthropic-native
or Responses API adapter is implemented; streaming/tools are not needed here.
Provider cancellation/interruption ownership remains future work.
