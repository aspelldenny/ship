# PHIẾU P001: Phiếu standard + docs discipline

> **ID format:** `P` + 3 chữ số. Counter ở `.phieu-counter`.
> **Filename:** `docs/ticket/P001-phieu-standard.md`.
> **Branch:** `chore/P001-phieu-standard`.

---

> **Loại:** Chore
> **Risk:** read-only (chỉ sửa docs + template, không đụng code)
> **Ưu tiên:** P1
> **Ảnh hưởng:** `CLAUDE.md`, `docs/ticket/*`, `docs/DISCOVERIES.md` (mới), `docs/CHANGELOG.md`, `.gitignore`
> **Dependency:** Không

---

## Context

### Vấn đề hiện tại

1. `docs/ticket/TEMPLATE.md` hiện tại quá sơ sài (32 dòng) — không ép Sếp grep/verify trước khi viết assumption. Phiếu `SHIP-NOTE-OBSIDIAN-INTEGRATION` (draft trong Obsidian vault) có 5 assumption chưa verify.
2. CLAUDE.md nói "Discovery Report filed" nhưng không chỉ đâu. Chat session bị clear = mất trace, Sếp không có nơi đọc lại để viết phiếu sau.
3. Không có convention naming phiếu (`P<NNN>-<slug>`). Shell function `phieu` và `.phieu-counter` đã setup (2026-04-24) nhưng CLAUDE.md chưa document.
4. Scenario chat-driven: Sếp chat với em → em phát hiện task cần làm → hiện tại không có 4-bước chuẩn để tự tạo phiếu mid-chat.
5. Phiếu taxonomy hiện chỉ có `read-only/mutating/destructive` (trục rủi ro). Thiếu trục `Feature/Bugfix/Hotfix/Chore` (kiểu thay đổi) — tarot có cả hai.

### Giải pháp

Port các điểm chín của tarot CLAUDE.md + TICKET_TEMPLATE (đã battle-tested qua Phase 2A-2C + Paper Pivot) về ship, điều chỉnh cho Rust context. Không port những phần project-specific (Tech stack, Pricing, Dev test, GitHub MCP token economy).

### Scope

- **CHỈ sửa**: `CLAUDE.md`, `docs/ticket/` (rewrite template), `docs/DISCOVERIES.md` (mới), `docs/CHANGELOG.md`, `.gitignore`
- **KHÔNG sửa**: Rust code (không đụng `src/`, `Cargo.toml`), `docs/ARCHITECTURE.md`, `docs/CONVENTION.md`, `docs/PROJECT.md`, `docs/ticket/PHASE-1.md` (historical — giữ nguyên)

---

## Verification Anchors — Sếp đã verify lúc viết phiếu

| # | Assumption | Verify bằng lệnh nào | Kết quả |
|---|-----------|---------------------|---------|
| 1 | Ship có `docs/ticket/TEMPLATE.md` (32 dòng, basic) | `cat docs/ticket/TEMPLATE.md \| wc -l` | ✅ 32 dòng |
| 2 | Ship chưa có `docs/ticket/TICKET_TEMPLATE.md` | `ls docs/ticket/` | ✅ KHÔNG tồn tại — phải tạo mới |
| 3 | Ship chưa có `docs/DISCOVERIES.md` | `ls docs/` | ✅ KHÔNG có — phải tạo mới |
| 4 | `.phieu-counter` tồn tại ở root, giá trị 0 | `cat .phieu-counter` | ✅ `0` |
| 5 | `.gitignore` đã có `.phieu-counter` entry | `grep phieu .gitignore` | ✅ (do `phieu-init` thêm) |
| 6 | Shell function `phieu` tìm `TICKET_TEMPLATE.md` trước, fallback `TEMPLATE.md` | `grep -A5 "TICKET_TEMPLATE" ~/.zshrc` | ✅ có lookup order |
| 7 | Tarot template dùng table Verification Anchors (# \| Assumption \| Verify \| Result) | `cat ~/tarot/docs/ticket/TICKET_TEMPLATE.md` | ✅ dòng 36-43 |
| 8 | Tarot CLAUDE.md có section "Tạo phiếu mid-chat" | `grep "mid-chat" ~/tarot/CLAUDE.md` | ✅ dòng 348+ |

---

## Nhiệm vụ

### Task 1: Viết lại TICKET_TEMPLATE

**File mới:** `docs/ticket/TICKET_TEMPLATE.md`
**File xoá:** `docs/ticket/TEMPLATE.md`

**Nội dung mới (port từ tarot, điều chỉnh Rust):**
- Header: ID format, filename, branch convention, mention `phieu` shell fn + sequential flow
- Frontmatter: Loại + Risk + Ưu tiên + Ảnh hưởng + Dependency
- Context: Vấn đề / Giải pháp / Scope
- **Verification Anchors table** (BẮT BUỘC, # | Assumption | Verify cmd | Result ✅/❌)
- Nhiệm vụ: Task structure với File / Tìm / Thay bằng / Lưu ý (Rust code fence)
- Files cần sửa (bảng) + Files KHÔNG sửa — verify only (bảng)
- Luật chơi (Constraints)
- Nghiệm thu: Automated (cargo build/test/clippy/fmt + no debug) / Manual / Regression / Docs Gate (CHANGELOG + ARCHITECTURE + CONVENTION + DISCOVERIES)

### Task 2: Tạo `docs/DISCOVERIES.md`

**File mới:** `docs/DISCOVERIES.md`

**Nội dung:**
- Header giải thích mục đích (Sếp đọc trước khi viết phiếu tiếp theo)
- Format entry template (`## P<NNN> — YYYY-MM-DD — <tiêu đề>` + 4 sections)
- Luật cứng: ghi vào file (không chat-only), chèn đầu file, không xoá entries cũ

### Task 3: Update `CLAUDE.md`

**File:** `CLAUDE.md`

**Thay đổi:**

1. **DoD item 7** — đổi "Discovery Report filed" → "Discovery Report ghi vào `docs/DISCOVERIES.md` (chèn đầu file, sau header — mới nhất lên trên)". Thêm dòng "Thiếu bất kỳ bước nào = phiếu CHƯA XONG."
2. **Discovery Report section** — thêm "Tại sao luật này tồn tại" (why-anchor), đổi format template sang `## P<NNN> — YYYY-MM-DD — <tiêu đề>`, enforce file destination, thêm luật cứng.
3. **Phiếu Classification** — giữ bảng Risk (read-only/mutating/destructive), thêm bảng Loại (Feature/Bugfix/Hotfix/Chore).
4. **Thay Git Workflow section** bằng section **"Phiếu Workflow — Naming & Counter"**:
   - Naming convention (`<type>/P<NNN>-<slug>`, commit prefix khớp type)
   - Shell function `phieu` (với các sub-commands)
   - Flow thủ công (sequential, không worktree) — 4 bước bash
   - **Sub-section "Tạo phiếu mid-chat"** — 4-bước bắt buộc khi Sếp chat-driven

### Task 4: Update CHANGELOG

Thêm entry `## Unreleased` + `### Changed (P001 — 2026-04-24, chore)` liệt kê 4 thay đổi.

### Task 5: Tạo phiếu file này

`docs/ticket/P001-phieu-standard.md` — file đang đọc.

### Task 6: Discovery Report

Thêm entry P001 vào `docs/DISCOVERIES.md` sau khi làm xong (chèn đầu file, sau header).

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `docs/ticket/TICKET_TEMPLATE.md` | **MỚI** — rich format |
| `docs/ticket/TEMPLATE.md` | **XOÁ** — thay bằng TICKET_TEMPLATE |
| `docs/DISCOVERIES.md` | **MỚI** — empty with header + format |
| `CLAUDE.md` | DoD item 7, Discovery Report section, Phiếu Classification, thay Git Workflow bằng Phiếu Workflow + Mid-chat |
| `docs/CHANGELOG.md` | Entry Unreleased — P001 chore |
| `docs/ticket/P001-phieu-standard.md` | **MỚI** — file phiếu này |
| `.gitignore` | Ignore `.phieu-counter` (đã có sẵn do phieu-init) |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `docs/ARCHITECTURE.md` | Không thay đổi code → không cần update Sections 1-9 |
| `docs/CONVENTION.md` | Không có gotcha/naming mới từ phiếu này |
| `docs/ticket/PHASE-1.md` | Historical record — giữ nguyên |
| `src/**/*.rs`, `Cargo.toml` | Phiếu docs-only, không đụng Rust |

---

## Luật chơi (Constraints)

1. KHÔNG sửa Rust code — phiếu này chỉ docs/template.
2. KHÔNG xoá `docs/ticket/PHASE-1.md` (historical).
3. KHÔNG port những phần tarot-specific: Tech Stack (ship dùng Cargo.toml), Sync Claude Web, Dev test, Pricing, Navigation.
4. GIỮ 2 trục taxonomy (Loại + Risk) — không thay thế, bổ sung.
5. Template mới phải hoạt động ngay với shell function `phieu` (đã có lookup order `TICKET_TEMPLATE.md` → `TEMPLATE.md`).

---

## Nghiệm thu

### Automated (CLAUDE.md DoD)
- [x] `cargo build --release` — zero warnings (docs-only, không có thay đổi code)
- [x] `cargo test` — all pass (không thay đổi test)
- [x] `cargo clippy -- -D warnings` — clean
- [x] `cargo fmt -- --check` — clean
- [x] No debug code — N/A

### Manual Testing
- [x] `ls docs/ticket/` — thấy `TICKET_TEMPLATE.md`, không còn `TEMPLATE.md`
- [x] `ls docs/` — thấy `DISCOVERIES.md`
- [x] `cat .phieu-counter` = `1`
- [x] Shell function `phieu` sẽ dùng `TICKET_TEMPLATE.md` cho phiếu tiếp theo (verify lookup order trong `~/.zshrc`)

### Regression
- [x] `docs/ticket/PHASE-1.md` vẫn tồn tại
- [x] `docs/ARCHITECTURE.md`, `docs/CONVENTION.md`, `docs/PROJECT.md` không thay đổi
- [x] Rust code không bị touch

### Docs Gate (CLAUDE.md ship)
- [x] `docs/CHANGELOG.md` — entry Unreleased P001 chore
- [x] `docs/ARCHITECTURE.md` — không cần update (không thay đổi module/struct/flow/constraint)
- [x] `docs/CONVENTION.md` — không cần update
- [x] `docs/DISCOVERIES.md` — Discovery Report P001 chèn đầu file
