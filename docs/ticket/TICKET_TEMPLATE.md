# PHIẾU P<NNN>: <Tên phiếu>

> **ID format:** `P` + 3 chữ số (P001, P042, P123). Số tiếp theo đọc từ `.phieu-counter`.
> **Filename:** `docs/ticket/P<NNN>-<slug>.md` (khớp tên branch, bỏ prefix `<type>/`).
> **Branch:** `<type>/P<NNN>-<slug>` với `<type>` ∈ {feat, fix, chore, docs, infra}.
> **Thường dùng qua `phieu <slug>`** (shell function tự tăng counter, tạo branch + file phiếu). Sequential không cần worktree. KHÔNG tự đặt số bằng tay.

---

> **Loại:** Feature / Bugfix / Hotfix / Chore
> **Risk:** read-only / mutating / destructive
> **Ưu tiên:** P0 / P1 / P2
> **Ảnh hưởng:** [files/modules chính bị ảnh hưởng]
> **Dependency:** [phiếu phải xong trước, hoặc "Không"]

---

## Context

### Vấn đề hiện tại
[Mô tả vấn đề hoặc feature cần làm]

### Giải pháp
[Mô tả approach — chọn gì, vì sao không chọn alternative]

### Scope
- CHỈ sửa: [liệt kê modules/files]
- KHÔNG sửa: [liệt kê ra ngoài scope]

---

## Verification Anchors — Sếp đã verify lúc viết phiếu

> **BẮT BUỘC:** Sếp PHẢI `grep` / `cargo check` code thật trước khi viết assumption.
> Thợ đọc bảng này để biết cái nào đã verified, cái nào chưa.

| # | Assumption | Verify bằng lệnh nào | Kết quả |
|---|-----------|---------------------|---------|
| 1 | [struct X tồn tại ở `src/foo/mod.rs`] | `grep "struct X" src/...` | ✅ Dòng 123 |
| 2 | [crate `dirs` đã có trong Cargo.toml] | `grep '^dirs' Cargo.toml` | ❌ chưa có — phải thêm |
| 3 | [CLI subcommand `note` chưa tồn tại] | `grep -r "note" src/cli.rs` | ✅ KHÔNG có — cần tạo mới |

**Nếu có ❌ → Sếp đã biết assumption sai và ghi rõ cách xử lý trong task tương ứng.**

---

## Nhiệm vụ

### Task 1: [Tên task]

**File:** `src/path/to/file.rs`

**Tìm:** [mô tả đoạn code thật — dùng nội dung text, KHÔNG dùng tên biến/constant nếu chưa verify]

**Thay bằng / Thêm:**
```rust
// nội dung mới
```

**Lưu ý:** [edge cases, cross-module interaction, constraint Rust đặc thù]

### Task 2: [...]

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `src/path/file.rs` | Task 1: mô tả ngắn |
| `Cargo.toml` | Thêm dep `xxx` |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `src/other.rs` | Hàm X vẫn work sau thay đổi |

---

## Luật chơi (Constraints)

1. [Ví dụ: KHÔNG break CLI interface hiện tại]
2. [Ví dụ: Additive only — không modify existing subcommand behavior]
3. [Ví dụ: Zero clippy warnings — theo CLAUDE.md DoD]

---

## Nghiệm thu

### Automated (CLAUDE.md DoD)
- [ ] `cargo build --release` — zero warnings
- [ ] `cargo test` — all pass
- [ ] `cargo clippy -- -D warnings` — clean
- [ ] `cargo fmt -- --check` — clean
- [ ] No debug code (`dbg!`, `println!` for debug, `todo!`, `unimplemented!`)

### Manual Testing
- [ ] [Test case 1 — end-to-end command chạy thành công]
- [ ] [Test case 2 — edge case]

### Regression
- [ ] [Subcommand X hiện tại vẫn hoạt động bình thường]

### Docs Gate (bắt buộc theo CLAUDE.md ship)
- [ ] `docs/CHANGELOG.md` — entry cho phiếu này
- [ ] `docs/ARCHITECTURE.md` — Sections 7-9 nếu module map / data structures / constraints thay đổi
- [ ] `docs/CONVENTION.md` — nếu có naming/gotcha mới
- [ ] `docs/DISCOVERIES.md` — Discovery Report chèn đầu file (sau header, mới nhất lên trên)
