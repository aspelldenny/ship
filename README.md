# Ship

Automated release workflow — test, commit, push, PR in one command.

## Install

```bash
cargo install --path .
```

## Usage

```bash
ship              # Full pipeline: test → docs-gate → version → changelog → commit → push → PR
ship check        # Pre-flight only (test + docs-gate, no commit)
ship init         # Auto-detect project, generate .ship.toml
ship --dry-run    # Simulate without side effects
```

## Configuration

Create `.ship.toml` in your project root (or run `ship init`):

```toml
name = "my-project"
base_branch = "main"

[test]
command = "pnpm test --run"

[docs_gate]
enabled = true
blocking = false
```

## Requirements

- [gh CLI](https://cli.github.com) for PR creation
- [docs-gate](https://github.com/aspelldenny/docs-gate) (optional)

## License

MIT
