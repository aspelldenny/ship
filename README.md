# Ship

**One command to rule the release.**

Ship automates the tedious release cycle — run tests, bump version, update changelog, commit, push, open PR — so you can focus on building. Built in Rust for speed and reliability.

```
$ ship
  Preflight ........ ok
  Test ............. ok (cargo test, 2.1s)
  Docs-gate ........ ok
  Version .......... 0.2.0 → 0.2.1
  Changelog ........ updated
  Commit ........... feat: add retry logic
  Push ............. origin/feat/retry
  PR ............... #42 created
```

## Status

Ship is at **v0.2.0** — functional but early. The core pipeline (`ship`, `ship check`, `ship init`) is battle-tested. Canary, deploy, learn, and MCP server are implemented and passing tests but have not been dogfooded on real projects yet. Expect rough edges.

| Feature | Status |
|---------|--------|
| `ship` (full pipeline) | Stable |
| `ship check` | Stable |
| `ship init` | Stable |
| `ship canary` | Implemented, needs real-world testing |
| `ship deploy` | Implemented, needs real-world testing |
| `ship learn` | Implemented, needs real-world testing |
| `ship note` | Implemented, needs real-world testing |
| `ship serve` (MCP) | Implemented, needs real-world testing |

## Install

```bash
cargo install --git https://github.com/aspelldenny/ship
```

Or build from source:

```bash
git clone https://github.com/aspelldenny/ship
cd ship
cargo install --path .
```

## Commands

| Command | What it does |
|---------|-------------|
| `ship` | Full pipeline: test → docs-gate → version → changelog → commit → push → PR |
| `ship check` | Pre-flight only — run tests + docs-gate, no commit |
| `ship init` | Auto-detect your stack, generate `.ship.toml` |
| `ship deploy` | Deploy to production (SSH, GitHub Actions, Render, Cargo, custom) |
| `ship canary` | Post-deploy health check (HTTP + Docker via SSH) |
| `ship learn` | Cross-project learnings — add, search, list, prune |
| `ship note` | Export a per-phiếu log into an Obsidian vault |
| `ship serve` | MCP server for Claude Code integration |

## Flags

```
--dry-run          Simulate without side effects
--skip-tests       Skip the test step
--skip-docs-gate   Skip docs-gate validation
--bump <level>     Force patch / minor / major
--no-pr            Commit + push only, skip PR creation
--config <path>    Custom config file
-v, --verbose      Verbose output
```

## Configuration

Run `ship init` to auto-detect your project and generate a config:

```bash
$ ship init
  Detected: Rust (Cargo.toml)
  Created: .ship.toml
```

Or create `.ship.toml` manually:

```toml
name = "my-project"
base_branch = "main"

[test]
command = "cargo test"

[docs_gate]
enabled = true
blocking = false

[deploy]
provider = "ssh"                    # ssh | github-actions | render | cargo | custom
ssh = "user@prod.example.com"
command = "cd /app && docker compose up -d"

[deploy.maintenance_mode]
on = "sudo systemctl start maintenance"
off = "sudo systemctl stop maintenance"
```

### Stack auto-detection

| Stack | Detected by | Default test command |
|-------|------------|---------------------|
| Rust | `Cargo.toml` | `cargo test` |
| Next.js | `next.config.*` | `pnpm test --run` |
| Flask | `requirements.txt` with flask | `python -m pytest tests/ -x` |
| Python | `pyproject.toml` | `python -m pytest` |
| Node.js | `package.json` | `npm test` |

## Deploy providers

**SSH** — Run commands on a remote server via SSH:
```bash
ship deploy --provider ssh --ssh user@host
```

**GitHub Actions** — Verify the latest workflow run passed:
```bash
ship deploy --provider github-actions
```

**Render** — Auto-deploys on push, canary verifies health:
```bash
ship deploy --provider render
```

**Cargo** — Publish to crates.io:
```bash
ship deploy --provider cargo
```

**Custom** — Run any command:
```bash
ship deploy --provider custom --command "./deploy.sh"
```

All providers support `--skip-canary` and optional post-deploy health checks.

## MCP server

Ship exposes an [MCP](https://modelcontextprotocol.io) server for AI-assisted workflows:

```bash
ship serve
```

Available tools:

| Tool | Description |
|------|-------------|
| `ship_check` | Run pre-flight checks |
| `ship_canary` | Health check deployed app |
| `ship_learn_add` | Record a learning |
| `ship_learn_search` | Search learnings |
| `ship_note_export` | Write a per-phiếu log to the Obsidian vault |

Add to your Claude Code MCP config:

```json
{
  "mcpServers": {
    "ship": {
      "command": "ship",
      "args": ["serve"]
    }
  }
}
```

## Learnings

Ship maintains a cross-project knowledge base in JSONL format:

```bash
ship learn add "Always run migrations before deploy" -t deploy,database
ship learn search "migration"
ship learn list --recent 5
ship learn prune
```

## Obsidian vault log (`ship note`)

Export a per-phiếu markdown log to an Obsidian vault under `10_Projects/<slug>/logs/`. The note captures the latest commit subject + body, `git diff --stat`, branch, commit URL, and (if available) the open PR URL.

```bash
# One-off: log current commit into vault
ship note --project tarot --ticket P042 --message "refactored credit check"

# Configure per-project for auto-logging on successful `ship check`:
# .ship.toml
# [obsidian]
# auto_log    = true
# vault_path  = "~/VibeNotes"      # optional, overrides OBSIDIAN_VAULT_PATH + default
# project_slug = "tarot"           # optional, overrides cwd dirname
```

Vault missing or write failure → warning on stderr, exit 0. Never fails `ship`. Writes are atomic (`tmp + rename`), safe under an `obsidian-git` auto-commit plugin.

## Requirements

- [gh CLI](https://cli.github.com) — for PR creation and GitHub Actions deploy
- [docs-gate](https://github.com/aspelldenny/docs-gate) — optional, validates docs are up-to-date

## License

[MIT](LICENSE)
