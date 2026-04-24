# Ship — Developer Guide

## Quick Reference

```bash
cargo build --release          # Build
cargo test                     # Run all tests
cargo clippy -- -D warnings    # Lint (zero warnings required)
cargo fmt -- --check           # Format check
```

## Role

Builder (thợ) nhận phiếu từ Architect (Sếp), implement trong scope.

## Definition of Done (per phiếu)

1. `cargo build --release` — zero warnings
2. `cargo test` — all pass
3. `cargo clippy -- -D warnings` — clean
4. No debug code (`dbg!`, `println!` for debug, `todo!`, `unimplemented!`)
5. Update `docs/CHANGELOG.md`
6. Update `docs/ARCHITECTURE.md` (Sections 7-9 if implementation changed)
7. Discovery Report ghi vào `docs/DISCOVERIES.md` (chèn đầu file, sau header — mới nhất lên trên)
8. Git commit with conventional message

**Thiếu bất kỳ bước nào = phiếu CHƯA XONG. Không báo cáo, không commit.**

## Hard Stops (DỪNG, hỏi Sếp)

- New module/file outside phiếu scope
- New dependency in Cargo.toml
- CLI interface changes (new subcommand, flag changes)
- Config schema changes (.ship.toml format)
- Refactor code not in phiếu

## Docs Gate 2 Tầng

### Tầng 1 (Hard — must update before commit)
- Function signatures, data flow
- CLI interface (flags, subcommands)
- Config schema (.ship.toml)
- Data structures (structs, enums)
- Module map
- Runtime behavior
- Known constraints

### Tầng 2 (Soft — update later)
- Variable names
- Error message wording
- Code comments
- Internal formatting

### Mapping
| Change Type | Target File | Section |
|------------|-------------|---------|
| New module/struct | ARCHITECTURE.md | 1. Module Map + 5. Data Structures |
| CLI flag change | ARCHITECTURE.md | 3. CLI Interface |
| Config change | ARCHITECTURE.md | 4. Config Schema |
| Pipeline step change | ARCHITECTURE.md | 2. Data Flow |
| New constraint | ARCHITECTURE.md | 9. Known Constraints |
| Feature complete | CHANGELOG.md | New entry |
| Convention/gotcha | CONVENTION.md | Relevant section |

## Discovery Report (mandatory per phiếu)

**Tại sao luật này tồn tại:** Sếp viết phiếu dựa trên docs + grep, nhưng có thể vẫn sai so với code thật. Nếu thợ phát hiện sai lệch mà không báo lại → Sếp tiếp tục viết phiếu sai → lỗi chồng lỗi.

**Trước khi báo "XONG", thợ PHẢI ghi Discovery Report vào `docs/DISCOVERIES.md`** (chèn đầu file, sau header — mới nhất lên trên, giống CHANGELOG). Format:

```markdown
## P<NNN> — YYYY-MM-DD — <tiêu đề>

### Assumptions trong phiếu — ĐÚNG:
- [từng assumption khớp code thật]

### Assumptions trong phiếu — SAI so với code thật:
- [mismatch X: phiếu ghi "...", thực tế "..." → đã sửa docs Y]
- [hoặc "Không có"]

### Edge cases phát hiện thêm:
- [edge cases phiếu không đề cập]
- [hoặc "Không có"]

### Docs đã cập nhật theo discoveries:
- [file nào sửa, sửa gì]
```

**Luật cứng:**
- Ghi vào file `docs/DISCOVERIES.md`, KHÔNG chỉ báo cáo trong chat (chat clear sau session).
- Nếu phiếu có assumption sai Tầng 1 (function signature, struct name, CLI flag) → thợ PHẢI cập nhật docs theo code thật ngay trong phiếu đó.
- Sếp đọc file này TRƯỚC KHI viết phiếu tiếp theo.

## Phiếu Classification

Phiếu có 2 trục độc lập — ghi cả hai trong frontmatter:

**Loại** (kiểu thay đổi — docs-gate check tự động):
| Loại | Khi nào |
|------|---------|
| **Feature** | Thêm capability mới |
| **Bugfix** | Sửa hành vi sai |
| **Hotfix** | Bugfix khẩn (P0, skip một số gate) |
| **Chore** | Refactor, docs, infra, CI, housekeeping |

**Risk** (mức rủi ro — quyết định review):
| Risk | Scope | Review |
|------|-------|--------|
| **read-only** | No side effects (queries, display, docs) | Self-review |
| **mutating** | Changes data/state, new code paths | Sếp review |
| **destructive** | Deletes data, breaks API, changes schema | Sếp review + test plan |

## Language Rules

- Vietnamese with Sếp (all communication)
- English in code, comments, commit messages
- English in documentation (docs/*.md)

## Phiếu Workflow — Naming & Counter

### Naming

- **ID format:** `P` + 3 chữ số (P001, P042, P123) — zero-padded.
- **Counter:** `.phieu-counter` ở repo root (local, per-machine; đã gitignored). Source of truth cho số tiếp theo — KHÔNG tự đặt số bằng tay.
- **Branch:** `<type>/P<NNN>-<slug>` với `<type>` ∈ `{feat, fix, chore, docs, infra}`.
- **Ticket file:** `docs/ticket/P<NNN>-<slug>.md` (khớp branch, bỏ prefix `<type>/`).
- **Slug:** kebab-case (chữ thường, số, dấu `-`). Ngắn gọn, mô tả intent.
- **Commit prefix:** khớp `<type>` của branch (`feat:`, `fix:`, `chore:`, `docs:`, `infra:`).

### Shell function `phieu`

Dùng shell function `phieu` để tạo phiếu mới đúng chuẩn (tự tăng counter, tạo branch, copy template, điền header):

```bash
cd ~/ship
phieu <slug>                  # default type=feat
phieu <type> <slug>           # type tường minh
phieu-list                    # list worktrees + next ID
phieu-done <P-slug>           # xoá worktree khi xong
```

> `phieu` mặc định tạo worktree + launch `claude` mới. Khi làm **tuần tự trong cùng cwd** (không parallel), có thể bỏ qua worktree — dùng flow thủ công bên dưới. Khi muốn làm **song song nhiều phiếu**, dùng `phieu` để tạo worktree độc lập.

### Flow thủ công (sequential, không worktree)

```bash
# 1. Tăng counter + tạo branch từ origin/main
git fetch origin main --quiet
n=$(($(cat .phieu-counter) + 1))
id=$(printf "P%03d" "$n")
slug=<your-slug>
type=<feat|fix|chore|docs|infra>
git checkout -b "$type/$id-$slug" origin/main
echo "$n" > .phieu-counter

# 2. Copy template + điền header
cp docs/ticket/TICKET_TEMPLATE.md "docs/ticket/$id-$slug.md"
# sed header: # PHIẾU $id: $slug

# 3. Viết nội dung phiếu (Context + Verification Anchors + Tasks + Nghiệm thu) TRƯỚC khi code

# 4. Implement → Nghiệm thu → CHANGELOG + DISCOVERIES → commit + push + PR
git add -A
git commit -m "$type: brief description (P<NNN>)"
git push origin "$type/$id-$slug"
gh pr create --title "$type: ... (P<NNN>)" --body "..."
```

### Tạo phiếu mid-chat (chat-driven)

Khi Sếp KHÔNG gõ `phieu <slug>` từ đầu mà chat với em để bàn ra phiếu ("em xem cái này có làm được không" → em phân tích → Sếp "ok triển đi") → em PHẢI tự tạo phiếu theo chuẩn, KHÔNG bắt Sếp thoát Claude.

**Trigger:** Sếp xác nhận muốn làm 1 task cụ thể (không còn brainstorm/research).

**4 bước bắt buộc (TRƯỚC khi code dòng nào):**

1. Đọc counter: `cat .phieu-counter` → tăng 1 → format `P<NNN>`
2. Đề xuất `<type>/P<NNN>-<slug>` → **hỏi Sếp confirm slug + type** (slug em chọn có thể sai intent)
3. Sau khi Sếp OK, chạy flow thủ công (4 lệnh trên) — tăng counter TRƯỚC khi `checkout -b`. Nếu checkout fail → rollback counter (`echo <old-n> > .phieu-counter`).
4. Viết Context + Verification Anchors + Tasks + Nghiệm thu vào phiếu TRƯỚC khi code — theo `TICKET_TEMPLATE.md`.

**KHÔNG tạo phiếu khi:** Sếp đang exploration/research/brainstorm — hỏi lại trước.

## Phase History

- Phase 1: MVP Ship Pipeline (done)
- Phase 2: Canary + Deploy (done — code complete, needs dogfooding)
- Phase 3: Intelligence — learnings, MCP server (done — code complete, needs dogfooding)
- Phase 3.5: Obsidian vault log — `ship note` + `[obsidian] auto_log` hook + `ship_note_export` MCP tool (P003, done — needs dogfooding)
- Phase 4: Ecosystem — AI review, Skill registration, GitHub Actions, Telegram (planned)
- Current focus: dogfooding on real projects (tarot, jarvis, media-rating)

## ship note — Obsidian vault log

`ship note` exports a per-phiếu markdown log to `<vault>/10_Projects/<slug>/logs/`. Source: `src/note/mod.rs`. Integration points:

- **Manual:** `ship note --project <slug> [--ticket <id>] [--message <text>] [--vault-path <path>]`. Prints written path on stdout.
- **Auto-log hook:** Set `.ship.toml [obsidian] auto_log = true`. After a successful `ship check`, a note is written. Opt-in default off.
- **MCP:** `ship_note_export` tool (params: `project_slug`, `ticket_id`, `message`, `vault_path`). Independent from `ship_check` — MCP callers must invoke explicitly.
- **Vault resolution priority:** CLI arg > `OBSIDIAN_VAULT_PATH` env > `.ship.toml [obsidian] vault_path` > `~/VibeNotes`.

**Design constraints:**
- Graceful: vault missing / write fail → warning on stderr, exit 0. Never fails ship.
- Atomic write (`tmp + rename`) — safe under `obsidian-git` auto-commit.
- Zero new deps — Vietnamese diacritic stripping via manual table in `src/note/mod.rs`; `~` expansion via local `shellexpand_with_home()` (mirrors `src/learn/mod.rs` pattern). Non-Vietnamese non-ASCII scripts fall through as `-` in slugify.
- Hook is CLI-level only — MCP `ship_check` does NOT auto-log (avoids surprise vault writes from AI agents).

## Gotchas

1. **Git command execution:** Use `std::process::Command::new("git")`, NOT git2 crate. Simpler, matches user's git config.
2. **gh CLI required:** PR creation depends on `gh` being installed and authenticated.
3. **Cross-platform paths:** Use `PathBuf` and `std::path`, never hardcode `/` or `\`.
4. **Test isolation:** Each test creates `tempfile::TempDir`, never touches real filesystem.
5. **Config defaults:** Every field in `.ship.toml` is optional. Missing = sensible default.
6. **Error propagation:** Use `thiserror` for error types, `?` operator everywhere. Never `unwrap()` in non-test code.
