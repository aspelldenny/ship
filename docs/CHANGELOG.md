# Changelog

## Unreleased

### Drafted (P002 — 2026-04-24, docs)
- `docs/ticket/P002-obsidian-note.md` — migration phiếu từ `~/VibeNotes/10_Projects/ship/tickets/SHIP-NOTE-OBSIDIAN-INTEGRATION.md` (vault draft) sang format TICKET_TEMPLATE mới. Nội dung: feature `ship note` subcommand — tự log phiếu hoàn thành vào Obsidian vault. 7 tasks, 11 Verification Anchors có grep results thật, 4 open questions chờ Sếp decide. Implementation CHƯA bắt đầu.

### Changed (P001 — 2026-04-24, chore)
- `docs/ticket/TICKET_TEMPLATE.md` replaces `TEMPLATE.md` — thêm Verification Anchors table (ép grep trước khi viết assumption), Task structure (File/Tìm/Thay/Lưu ý), Files tables (sửa + verify-only), Regression checklist, Docs Gate section
- `docs/DISCOVERIES.md` mới — đích đến bắt buộc cho Discovery Report (trước chỉ nói "filed", giờ là file cụ thể)
- `CLAUDE.md` — thêm section "Phiếu Workflow — Naming & Counter" (P<NNN> convention, `.phieu-counter`, `phieu` shell fn, flow thủ công), section "Tạo phiếu mid-chat" (4-bước khi Sếp chat-driven), taxonomy 2-trục (Loại vs Risk), Discovery Report mandate với "tại sao"
- `.gitignore` — ignore `.phieu-counter` (local, per-machine)

## 0.2.0 (2026-04-04)

### Added
- `ship canary` — HTTP health check with latency + Docker container status via SSH
- `ship deploy` — 5 providers: ssh, github-actions, render, cargo, custom
  - Maintenance mode toggle, post-deploy canary integration
- `ship learn` — Cross-project learnings in JSONL
  - Subcommands: add (with tags), search, list, prune
- `ship serve` — MCP server exposing 4 tools via rmcp stdio transport
  - ship_check, ship_canary, ship_learn_add, ship_learn_search
- DeployConfig in .ship.toml (provider, ssh, command, maintenance_mode)

### Improved
- Binary now 4.7MB (added reqwest TLS for HTTP health checks)
- 34 tests (up from 21)

## 0.1.0 (2026-04-04)

### Added
- Project scaffold: Cargo.toml, module structure, docs
- `ship` — full pipeline: preflight → test → docs-gate → version → changelog → commit → push → PR
- `ship check` — pre-flight only (test + docs-gate, no commit)
- `ship init` — auto-detect project stack, generate .ship.toml
- Project detection: Rust, Next.js, Flask, Python, Node
- Config system (.ship.toml with sensible defaults)
- docs-gate integration (runs if binary in PATH)
- CI/CD: GitHub Actions (test 3 OS + release 4 targets)
- SKILL.md: Claude Code skill for /ship command
- 21 unit tests
