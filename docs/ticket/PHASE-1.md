# Phase 1 — MVP Ship Pipeline

**Type:** `mutating`
**Priority:** P0
**Estimate:** 3-5 sessions

## Goal
Ship full pipeline: preflight → test → docs-gate → version → changelog → commit → push → PR

## Tickets

### 1.1 — Project Detection + Config
- [ ] `detect.rs`: auto-detect stack from filesystem
- [ ] `config.rs`: load `.ship.toml`, merge with defaults
- [ ] `ship init`: generate config from detection
- [ ] Tests: detection accuracy for all 5 stack types

### 1.2 — Preflight + Test Runner
- [ ] `preflight.rs`: branch check, git status, diff stats
- [ ] `test.rs`: resolve test command from stack, run, capture output
- [ ] Gate logic: abort on test failure
- [ ] Tests: mock git commands, verify gates

### 1.3 — docs-gate Integration
- [ ] `docs_gate.rs`: check if binary exists, run, parse result
- [ ] Configurable blocking (warn vs fail)
- [ ] Tests: mock docs-gate binary

### 1.4 — Version + Changelog
- [ ] `version.rs`: read/write VERSION, Cargo.toml, package.json
- [ ] Auto-bump from diff size
- [ ] `changelog.rs`: parse git log, group by type, generate entry
- [ ] Tests: version bump logic, changelog formatting

### 1.5 — Commit + Push + PR
- [ ] `commit.rs`: stage + commit with conventional message
- [ ] `push.rs`: push with upstream tracking
- [ ] `pr.rs`: create/update PR via `gh` CLI
- [ ] PR body template with test results + docs-gate status
- [ ] Tests: command construction, PR body generation

### 1.6 — Pipeline Orchestrator
- [ ] `pipeline/mod.rs`: run steps sequentially with gates
- [ ] `--dry-run` mode (simulate without side effects)
- [ ] `--skip-tests`, `--skip-docs-gate` flags
- [ ] Summary output (steps, duration, PR URL)
- [ ] Integration test: full pipeline with mock git

## Done When
- `ship` runs full pipeline on a test project
- `ship check` runs preflight only
- `ship init` generates valid `.ship.toml`
- All tests pass, clippy clean
- Docs updated (ARCHITECTURE sections 7-9, CHANGELOG)
