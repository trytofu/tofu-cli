# Tofu CLI

A terminal-first webhook relay for local development. Give webhook providers one stable URL, forward events to localhost, inspect deliveries, and replay requests.

> Tofu is in early access. Some features are still being tested.

## Install

```bash
curl -fsSL https://trytofu.dev/install | sh
```

## Quick start

```bash
tofu login
tofu hooks create stripe --name "Stripe"
tofu hooks url stripe
tofu targets set local "http://127.0.0.1:3000/api/webhooks/stripe" --hook stripe
tofu watch stripe
```

1. `tofu login` opens browser-based device approval.
2. `tofu hooks create` gives you a stable URL to paste into the provider dashboard.
3. `tofu targets set` points that hook at your local server.
4. `tofu watch` streams incoming events and delivery results.

## Commands

| Command | Description |
|---------|-------------|
| `tofu login` | Authenticate via browser device approval |
| `tofu hooks create <slug>` | Create a named hook with a stable provider URL |
| `tofu hooks url <slug>` | Print the provider URL |
| `tofu hooks list` | List hooks in the current workspace |
| `tofu targets set <name> <url> --hook <slug>` | Set a forwarding target |
| `tofu targets list --hook <slug>` | List targets for a hook |
| `tofu watch <slug>` | Stream live events and delivery status |
| `tofu replay latest --hook <slug>` | Replay the most recent event |
| `tofu events list --hook <slug>` | List recent events |
| `tofu events expire <event-id>` | Remove stored payload immediately |
| `tofu health` | Check API connectivity |
| `tofu config show` | Show current config (token masked) |

All commands accept `--json` for machine-readable output.

## Configuration

Config lives at `~/.config/tofu/config.toml`:

```toml
api_base_url = "https://api.trytofu.dev"
token = "tofu_pat_xxxxxxxx"
```

## What Tofu does

- Accepts webhooks from providers (Stripe, GitHub, Resend, etc.) at a stable URL
- Stores request body and headers temporarily for replay
- Forwards events to one or more local/dev targets
- Streams live delivery status via `tofu watch`
- Replays retained events after code changes

Tofu is not an ngrok replacement. Use ngrok, localhost, staging, or a preview URL as your target. Tofu gives the provider a stable URL so you don't have to reconfigure it when your local setup changes.

## Docs

Full quickstart and provider examples: [trytofu.dev/docs](https://trytofu.dev/docs)

## License

MIT
