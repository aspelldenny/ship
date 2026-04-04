# Ship — Rust CLI for Automated Release Workflow

## Vision

Single command that takes code from "done" to "PR created + verified". Replaces manual: test → commit → push → PR → monitor deploy.

Built for solo developers managing multiple projects (Tarot, Jarvis, Media Rating, docs-gate). One binary, zero runtime dependencies, works everywhere.

## Problem

Current workflow is manual and error-prone:
1. Run tests (sometimes forget)
2. Update CHANGELOG (sometimes skip)
3. Commit with inconsistent messages
4. Push, create PR manually
5. Deploy, hope nothing breaks
6. No post-deploy verification

## Solution

```bash
ship              # Full pipeline: test → review → version → changelog → commit → push → PR
ship check        # Pre-flight only (tests + review, no commit)
ship canary       # Post-deploy health check
ship learn        # Manage cross-project learnings
ship serve        # MCP server for Claude integration
ship init         # Auto-detect project, generate .ship.toml
```

## Target Projects

| Project | Stack | Deploy | Specifics |
|---------|-------|--------|-----------|
| Tarot (Soul Signature) | Next.js + Docker | VPS GitHub Actions | Health check, maintenance mode |
| Jarvis | Python + Docker | VPS SSH manual | Telegram bot, check webhook |
| Media Rating | Flask + Render | Render auto-deploy | Check Render status |
| docs-gate | Rust | crates.io + GitHub Release | cargo publish |
| Future Rust tools | Rust | GitHub Release | Cross-platform binaries |

## Core Principles

1. **One command, full pipeline** — no partial states
2. **Gate-based** — each step must pass before next (learned from gstack)
3. **Project-agnostic** — detect stack, adapt commands
4. **Fail-safe** — never push broken code, never deploy without verify
5. **Cross-project learnings** — mistakes in Tarot prevent same mistake in Jarvis

## Features

### Phase 1 — MVP Ship Pipeline
- [x] Project detection (Next.js, Flask, Rust, Python)
- [x] Test runner (auto-detect: pytest, vitest, cargo test, pnpm test)
- [x] docs-gate integration (if .docs-gate.toml exists)
- [x] Version bump (auto-decide based on diff size)
- [x] CHANGELOG auto-generate from commits
- [x] Bisectable commits (group by type)
- [x] Push + PR creation (gh CLI)
- [x] Config system (.ship.toml)

### Phase 2 — Canary + Deploy
- [ ] Post-deploy health check (HTTP, Docker, process)
- [ ] Deploy trigger (SSH, Render API, cargo publish)
- [ ] Maintenance mode toggle (Nginx)
- [ ] Rollback on failure

### Phase 3 — Intelligence
- [ ] AI code review (Claude API via OpenRouter)
- [ ] Cross-project learnings (JSONL)
- [ ] MCP server (4+ tools)
- [ ] Watch mode

### Phase 4 — Ecosystem
- [ ] Claude Code skill (SKILL.md)
- [ ] GitHub Actions integration
- [ ] Notification (Telegram bot → Jarvis)

## Non-Goals
- Not a CI/CD replacement (complements GitHub Actions)
- Not a project scaffolder (use existing templates)
- Not multi-user (single developer tool)
- No GUI (CLI only)

## Tech Stack

| Component | Choice | Why |
|-----------|--------|-----|
| Language | Rust (Edition 2024) | Single binary, fast, Sếp's ecosystem |
| CLI | clap 4.x (derive) | Same as docs-gate, proven pattern |
| Config | toml 0.8 + serde | Same as docs-gate |
| Git | git2 or Command::new("git") | Native git operations |
| HTTP | reqwest | Health checks, API calls |
| MCP | rmcp 0.8 | Same as docs-gate, proven pattern |
| Async | tokio (current_thread) | Watch mode, MCP server |
| Process | std::process::Command | Run tests, git, gh CLI |
| Filesystem | notify 8.x | Watch mode (Phase 3) |
| JSON | serde_json | Learnings, PR body |
| Date | chrono 0.4 | CHANGELOG dates, version timestamps |
| Regex | regex 1.x | Commit parsing, stack detection |

## Revenue Model
None — personal infra tool, open-source (MIT).

## Success Metrics
- Ship time: < 2 minutes (from command to PR URL)
- Zero manual steps between "code done" and "PR created"
- Zero broken deploys (canary catches before users)
- Learnings prevent repeat mistakes across projects
