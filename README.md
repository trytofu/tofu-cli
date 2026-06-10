# Tofu CLI

A terminal-first webhook relay for local development. Give webhook providers one stable URL, forward events to your dev or preview services, inspect requests, and replay them after code changes.

> Tofu is in early access. Some features are still being tested.

## Install

```bash
curl -fsSL https://trytofu.dev/install | sh
```

## Homebrew

```bash
brew install trytofu/tap/tofu
```

## What you need

- A Tofu account
- A local, staging, or preview URL that your webhook provider can reach
- One workspace selected in the CLI

## Quick start

```bash
tofu login

tofu workspaces list
tofu workspaces use <workspace-slug>

tofu hooks create stripe --name "Stripe"
tofu targets set local "http://127.0.0.1:3000/api/webhooks/stripe" --hook stripe
tofu hooks url stripe

tofu events list --hook stripe
tofu replay latest --hook stripe
```

1. `tofu login` opens browser-based device approval and saves your token.
2. `tofu workspaces use` chooses where hooks and targets are managed.
3. `tofu hooks create` creates a stable provider URL.
4. `tofu targets set` forwards matching events to your service.
5. `tofu hooks url` prints the URL to paste into Stripe, GitHub, Resend, or another provider.
6. `tofu events list` and `tofu replay` let you inspect and resend retained events.

`http://127.0.0.1:3000/...` only works if your local service is reachable from the Tofu API. In many setups that means using a tunnel, staging URL, or preview deployment as the target URL.

## Commands

| Command | Description |
|---------|-------------|
| `tofu login` | Authenticate via browser device approval |
| `tofu login --token <token>` | Authenticate with a personal API token |
| `tofu logout` | Clear the saved token |
| `tofu whoami` | Show the authenticated user |
| `tofu usage` | Show plan usage and remaining limits |
| `tofu workspaces list` | List workspaces you can access |
| `tofu workspaces use <slug>` | Set the active workspace |
| `tofu workspaces create <name-or-slug>` | Create a workspace |
| `tofu workspaces members list` | List active workspace members |
| `tofu workspaces members add <email>` | Add a member to the active workspace |
| `tofu hooks create <slug>` | Create a named hook with a stable provider URL |
| `tofu hooks url <slug>` | Print the provider URL |
| `tofu hooks list` | List hooks in the current workspace |
| `tofu hooks status <slug>` | Show hook details and targets |
| `tofu targets add <name> <url> --hook <slug>` | Add a forwarding target |
| `tofu targets set <name> <url> --hook <slug>` | Set a forwarding target |
| `tofu targets list --hook <slug>` | List targets for a hook |
| `tofu targets enable <name> --hook <slug>` | Enable a target |
| `tofu targets disable <name> --hook <slug>` | Disable a target |
| `tofu targets delete <name> --hook <slug>` | Delete a target |
| `tofu events list --hook <slug>` | List recent events |
| `tofu events latest --hook <slug>` | Show the latest event for a hook |
| `tofu events show <event-id>` | Show event headers, body preview, and deliveries |
| `tofu events expire <event-id>` | Remove stored payload immediately |
| `tofu replay <event-id>` | Replay an event to enabled targets |
| `tofu replay <event-id> --target <name>` | Replay an event to one target |
| `tofu replay latest --hook <slug>` | Replay the most recent event for a hook |
| `tofu health` | Check API connectivity |
| `tofu config show` | Show current config (token masked) |

All commands accept `--json` for machine-readable output. Use `--api-base-url <url>` to temporarily point the CLI at another API.

Live event streaming with `tofu watch` is not included in this rewrite yet. Until it lands, use `tofu events list --hook <slug>` or `tofu events latest --hook <slug>` after sending a test webhook.

## Configuration

Config lives at `~/.config/tofu/config.toml`:

```toml
api_base_url = "https://api.trytofu.dev"
token = "tofu_pat_xxxxxxxx"
```

You can override the config location with `TOFU_CONFIG_PATH`, which is useful for scripts and tests.

## What Tofu does

- Accepts webhooks from providers (Stripe, GitHub, Resend, etc.) at a stable URL
- Stores request body and headers temporarily for replay
- Forwards events to one or more local/dev targets
- Replays retained events after code changes

Tofu is not an ngrok replacement. Use ngrok, staging, a preview URL, or another reachable endpoint as your target. Tofu gives the provider a stable URL so you do not have to reconfigure provider webhooks when your target changes.

## Docs

Full quickstart and provider examples: [trytofu.dev/docs](https://trytofu.dev/docs)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for contribution guidelines, including
the policy for AI-assisted submissions.

## License

MIT
