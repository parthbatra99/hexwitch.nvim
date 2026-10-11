# Compatible API connection

The CLI and playground host use one OpenAI-compatible Chat Completions transport.
Configure API root, model and credential environment-variable name:

```sh
hexwitch playground --base-url http://slingring.procol.tech \
  --api-key-env SLINGRING_API_KEY --model FAST_SMALLER
```

`OPENAI_BASE_URL` is the API-root environment override. The host appends
`/chat/completions`; it does not invent a `/v1` path. Legacy `--provider` aliases
select convenience defaults only. Credentials are read server-side on explicit
live requests. Browser API settings show only key variable name/presence.

Output modes: `json_schema` (strict server schema), `json_object`, `none`. Local
parsing and intent validation remain strict in every mode. Non-schema modes add
schema guidance to the system prompt. No automatic retry, format downgrade or
sampling parameter is injected.

HTTP and HTTPS are accepted. HTTP warns that credentials/prompts can be intercepted
or modified. URL credentials/query/fragments and complete endpoint paths are
rejected. Curl receives credentials and payload through stdin, not argv or files.
Redirects are disabled. Payload/response/time limits and refusal/truncation/status
checks remain enforced, with redacted transport failures.

Offline TCP tests exercise real HTTP transport, custom API paths, all output modes,
key environment selection and playground clarification continuation. These prove
transport behavior, not live semantic quality. Live small-model observations and
limits are recorded in [engine-playground.md](engine-playground.md).

Editor binary discovery/configuration is removed. The host is started explicitly
from the Rust binary. No native-provider SDK, Responses API, automatic cancellation
or editor wrapper is included.
