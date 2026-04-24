# Architecture — Ship CLI

## 1. Module Map

```
src/
├── main.rs                    # Entry point: clap CLI + tokio runtime
├── config.rs                  # .ship.toml loading, defaults, project detection
├── detect.rs                  # Auto-detect project stack (Next.js, Flask, Rust, Python)
├── pipeline/
│   ├── mod.rs                 # Pipeline orchestrator (run steps sequentially with gates)
│   ├── preflight.rs           # Step 1: git status, branch check, uncommitted changes
│   ├── test.rs                # Step 2: run test suite (auto-detect framework)
│   ├���─ docs_gate.rs           # Step 3: run docs-gate if available
│   ├── review.rs              # Step 4: AI code review (Phase 3, stub for now)
│   ├── version.rs             # Step 5: bump VERSION file
│   ├── changelog.rs           # Step 6: generate CHANGELOG entry from commits
│   ├── commit.rs              # Step 7: bisectable commits (infra → logic → version)
│   ├── push.rs                # Step 8: git push -u origin <branch>
│   └── pr.rs                  # Step 9: gh pr create with structured body
├── canary/
│   ├── mod.rs                 # Canary orchestrator
│   ├── http.rs                # HTTP health check (status code, response time)
│   ├── docker.rs              # Docker container status check (SSH)
│   └── process.rs             # Process liveness check (PID, port)
├── learn/
│   ├── mod.rs                 # Learnings manager
│   ├── store.rs               # JSONL read/write/search
│   └── display.rs             # Format learnings for terminal
├── mcp/
│   ├── mod.rs                 # MCP module entry
│   ├── server.rs              # ShipServer + ServerHandler impl
│   └── tools.rs               # Tool parameter types + routing
├── note/
│   └── mod.rs                 # Obsidian vault log export (ship note)
├── output.rs                  # Terminal output formatting (colors, status icons)
└── error.rs                   # Error types (ShipError enum)
```

## 2. Data Flow

### Ship Pipeline (main flow)

```
ship [--dry-run] [--skip-tests] [--skip-review]
  │
  ├─ Load config (.ship.toml or defaults)
  ├─ Detect project stack (detect.rs)
  │
  ├─ Step 1: PREFLIGHT (preflight.rs)
  │   ├─ Check not on main/master
  │   ├�� Check git status (uncommitted changes → auto-stage)
  ��   ├─ Read diff stats (files changed, insertions, deletions)
  │   └─ GATE: abort if on protected branch
  │
  ├─ Step 2: TEST (test.rs)
  │   ├─ Auto-detect test command from stack
  │   ��─ Run tests, capture stdout/stderr
  │   ├─ Parse exit code
  │   └─ GATE: abort if tests fail (unless --skip-tests)
  │
  ├─ Step 3: DOCS-GATE (docs_gate.rs)
  │   ├─ Check if docs-gate binary exists in PATH
  ��   ├─ Run `docs-gate --all`
  │   ├─ Parse exit code
  │   └─ GATE: warn if fails (non-blocking, configurable)
  │
  ├─ Step 4: REVIEW (review.rs) [Phase 3]
  ��   ├─ Call AI API with diff
  │   ├─ Parse findings
  │   └─ GATE: block on critical findings
  │
  ├─ Step 5: VERSION (version.rs)
  │   ├─ Read current VERSION file (or Cargo.toml, package.json)
  │   ├─ Calculate bump level from diff size:
  │   ���   - <50 lines → patch (0.0.1)
  │   │   - 50-500 lines → minor (0.1.0)
  │   │   - >500 lines → ask user
  │   ├─ Write new version
  │   └─ GATE: none (informational)
  │
  ├─ Step 6: CHANGELOG (changelog.rs)
  │   ├─ Read git log (branch commits vs base)
  │   ├─ Group commits by type (feat/fix/refactor/docs/chore)
  │   ├─ Generate markdown entry with date
  │   ├─ Prepend to CHANGELOG.md
  │   └─ GATE: none
  │
  ├─ Step 7: COMMIT (commit.rs)
  │   ├─ Stage all changes
  │   ├─ Create commit with conventional message
  │   ├─ Include Co-Authored-By if AI assisted
  │   └─ GATE: none
  │
  ├─ Step 8: PUSH (push.rs)
  │   ├─ Check if remote tracking exists
  │   ├─ Push with -u flag
  ��   └─ GATE: abort on push failure
  │
  └─ Step 9: PR (pr.rs)
      ├─ Check if PR already exists (gh pr view)
      ├─ If exists: update body
      ├─ If new: create with structured body
      │   ├─ ## Summary (commits grouped by type)
      │   ├─ ## Test Results (pass/fail count)
      │   ├─ ## Docs Gate (pass/warn/fail)
      │   ├─ ## Review (findings if any)
      │   └─ ## Test Plan (checklist)
      └─ Output: PR URL
```

### Canary Flow

```
ship canary [--url <url>] [--docker <container>] [--timeout <secs>]
  │
  ├─ Load config (.ship.toml canary section)
  ├─ HTTP health check (GET url, expect 200, measure latency)
  ├─ Docker check (ssh → docker ps → container running?)
  ├─ Process check (port open? PID alive?)
  └─ Report: ✅ all green / ❌ issues + details
```

### Learn Flow

```
ship learn add "message"           # Append to learnings.jsonl
ship learn search "keyword"        # Fuzzy search
ship learn list [--recent N]       # Show recent
ship learn prune                   # Interactive cleanup
```

### MCP Server Flow

```
ship serve
  │
  ├─ Start rmcp stdio server
  ├─ Expose tools:
  │   ├─ ship_check      → run preflight + test + docs-gate (no commit)
  │   ├─ ship_full        → run full pipeline
  │   ├─ ship_canary      → run health checks
  │   ├─ ship_learn_add   → add learning
  │   └─ ship_learn_search → search learnings
  └─ Await client disconnect
```

## 3. CLI Interface

```
ship                              # Full pipeline (default)
ship check                        # Pre-flight only (test + docs-gate, no commit)
ship canary                       # Post-deploy health check
ship learn <subcommand>           # Learnings management
ship note                         # Export per-phiếu log to Obsidian vault
ship serve                        # MCP server mode
ship init                         # Generate .ship.toml from project detection

Options (global):
  --config <path>                 # Custom config file
  --verbose                       # Show all step output
  --dry-run                       # Simulate without side effects
  --skip-tests                    # Skip test step
  --skip-docs-gate                # Skip docs-gate step

Options (ship):
  --bump <patch|minor|major>      # Override version bump
  --no-pr                         # Commit + push only, no PR
  --base <branch>                 # Override base branch

Options (canary):
  --url <url>                     # Health check URL
  --docker <container>            # Docker container name
  --timeout <secs>                # Health check timeout (default: 30)
  --ssh <user@host:port>          # SSH for Docker check

Options (note):
  --project <slug>                # Project slug (overrides config + cwd dirname)
  --ticket <id>                   # Ticket ID for frontmatter
  --message <text>                # Free-form learnings line (section omitted if absent)
  --vault-path <path>             # Vault root (overrides env OBSIDIAN_VAULT_PATH + config)
```

## 4. Config Schema (.ship.toml)

```toml
# Project identity
name = "tarot"                          # Project name (auto-detected)
stack = "nextjs"                        # auto | nextjs | flask | rust | python
base_branch = "main"                    # main | master | develop

# Test
[test]
command = "pnpm test --run"             # Override auto-detected test command
timeout_secs = 300                      # Test timeout (default: 5 min)

# Docs gate
[docs_gate]
enabled = true                          # Run docs-gate check
blocking = false                        # true = fail pipeline on docs-gate fail

# Version
[version]
file = "VERSION"                        # VERSION | Cargo.toml | package.json
strategy = "auto"                       # auto | manual | semver
auto_thresholds = { patch = 50, minor = 500 }

# Changelog
[changelog]
file = "docs/CHANGELOG.md"             # Path to CHANGELOG
style = "grouped"                       # grouped (by type) | flat

# PR
[pr]
template = "default"                    # default | minimal | detailed
draft = false                           # Create as draft PR
labels = ["ship"]                       # Auto-add labels

# Canary (Phase 2)
[canary]
url = "https://www.soulsign.me"         # Health check URL
docker_container = "soulsign-nextjs-1"  # Container to check
ssh = "deploy@103.167.150.178:1994"     # SSH for Docker check
timeout_secs = 30                       # Health check timeout
checks = ["http", "docker"]             # Which checks to run

# Deploy (Phase 2)
[deploy]
provider = "github-actions"             # github-actions | ssh | render | cargo
command = ""                            # Custom deploy command
maintenance_mode = true                 # Toggle maintenance during deploy

# AI Review (Phase 3)
[review]
enabled = false                         # AI review (requires API key)
provider = "openrouter"                 # openrouter | anthropic
model = "anthropic/claude-sonnet-4"     # Model for review

# Learnings
[learn]
dir = "~/.ship/learnings"              # Global learnings directory
project_dir = ".ship/learnings"         # Per-project learnings

# Obsidian vault log (ship note)
[obsidian]
auto_log = false                        # Post-success hook in `ship check`: auto-write note
vault_path = "~/VibeNotes"             # Override env OBSIDIAN_VAULT_PATH. Default: ~/VibeNotes
project_slug = "tarot"                  # Override auto-detect (else cwd dirname)
```

## 5. Data Structures

```rust
// Pipeline result
pub struct PipelineResult {
    pub steps: Vec<StepResult>,
    pub duration: Duration,
    pub pr_url: Option<String>,
}

pub struct StepResult {
    pub name: String,           // "preflight", "test", "docs_gate", etc.
    pub status: StepStatus,
    pub duration: Duration,
    pub output: Option<String>, // Captured stdout/stderr
}

pub enum StepStatus {
    Pass,
    Fail(String),       // Error message
    Warn(String),       // Warning (non-blocking)
    Skip(String),       // Skipped (--skip-tests, etc.)
}

// Project detection
pub enum ProjectStack {
    NextJs,             // package.json + next.config.*
    Flask,              // requirements.txt + flask in deps
    Rust,               // Cargo.toml
    Python,             // requirements.txt or pyproject.toml
    Node,               // package.json (generic)
    Unknown,
}

// Version
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub micro: Option<u32>,     // 4th digit (gstack style)
}

pub enum BumpLevel {
    Micro,      // 0.0.0.1 (< 50 lines)
    Patch,      // 0.0.1   (50-500 lines)
    Minor,      // 0.1.0   (500+ lines or feature)
    Major,      // 1.0.0   (breaking change)
}

// Canary
pub struct CanaryResult {
    pub checks: Vec<HealthCheck>,
    pub overall: CanaryStatus,
}

pub struct HealthCheck {
    pub name: String,           // "http", "docker", "process"
    pub status: CanaryStatus,
    pub latency_ms: Option<u64>,
    pub details: Option<String>,
}

pub enum CanaryStatus {
    Healthy,
    Degraded(String),
    Down(String),
}

// Learnings
pub struct Learning {
    pub id: String,             // UUID
    pub timestamp: DateTime<Utc>,
    pub project: String,        // "tarot", "jarvis", etc.
    pub message: String,
    pub tags: Vec<String>,      // ["deploy", "docker", "prisma"]
}

// Obsidian vault note (ship note)
pub struct NoteOptions {
    pub project: Option<String>,      // slug; fallback to config/cwd dirname
    pub ticket: Option<String>,       // ticket id for frontmatter
    pub message: Option<String>,      // learnings body; section omitted if None
    pub vault_path: Option<String>,   // override env + config
}

pub enum NoteOutcome {
    Written(PathBuf),                 // path to the written note
    Skipped(String),                  // graceful skip reason (vault missing, write fail)
}
```

## 6. Module Dependencies

```
main.rs
  ├─ config.rs (load .ship.toml)
  ├─ detect.rs (identify project stack)
  ├─ pipeline/mod.rs (orchestrate steps)
  │   ├─ preflight.rs  → git commands
  │   ├─ test.rs       → Command::new(test_cmd)
  │   ├─ docs_gate.rs  → Command::new("docs-gate")
  │   ├─ review.rs     → reqwest (Phase 3)
  │   ├─ version.rs    → fs read/write
  │   ├─ changelog.rs  → git log + fs write
  │   ├─ commit.rs     → git commands
  │   ├─ push.rs       → git push
  │   └─ pr.rs         → Command::new("gh")
  ├─ canary/mod.rs
  │   ├─ http.rs       → reqwest
  │   ├─ docker.rs     → Command::new("ssh")
  │   └─ process.rs    → Command::new("curl") / TcpStream
  ��─ learn/mod.rs
  │   ├─ store.rs      → fs (JSONL)
  │   └─ display.rs    → terminal output
  ├─ mcp/mod.rs
  │   ├─ server.rs     → rmcp
  │   └─ tools.rs      → serde + schemars
  ├─ note/mod.rs       → fs atomic write + git/gh Command
  ├─ output.rs         → stdout formatting
  └─ error.rs          → ShipError enum
```

## 7. Implementation Notes

### Stack Detection Algorithm (detect.rs)
```
1. Check Cargo.toml exists → Rust
2. Check package.json exists:
   a. Has "next" in dependencies → NextJs
   b. Else → Node
3. Check requirements.txt exists:
   a. Contains "flask" → Flask
   b. Else → Python
4. Check pyproject.toml exists → Python
5. Else → Unknown
```

### Test Command Resolution (test.rs)
```
NextJs   → "pnpm test --run" or "npm test"
Flask    → "python -m pytest tests/ -x"
Rust     → "cargo test"
Python   → "python -m pytest"
Node     → "npm test"
Unknown  → ask user or skip
```

### Commit Grouping Strategy (commit.rs)
Learned from gstack: commits should be bisectable.
```
Order: infrastructure → models/services → controllers/views → version+changelog
Each commit independently valid (no broken imports).
```

### CHANGELOG Generation (changelog.rs)
Learned from gstack: user-centric, not implementation details.
```
feat: → "Added: ..."
fix:  → "Fixed: ..."
refactor: → "Improved: ..."
docs: → "Documentation: ..."
chore: → (skip in CHANGELOG)
```

### PR Body Template (pr.rs)
Learned from gstack + tarot conventions:
```markdown
## Summary
- [grouped commit descriptions]

## Test Results
- ✅ X tests passed (Y skipped)
- Duration: Zs

## Docs Gate
- ✅ CHANGELOG: up to date
- ✅ ARCHITECTURE: 9/9 sections

## Test Plan
- [ ] Verify on staging
- [ ] Check production after deploy

---
Generated by [ship](https://github.com/aspelldenny/ship)
```

## 8. Runtime Behavior

- **Binary size target:** < 10MB (release, stripped)
- **Startup:** < 5ms
- **Full pipeline:** < 2 minutes (excluding test runtime)
- **MCP server:** Persistent process on stdio (same as docs-gate)
- **Learnings:** File-locked JSONL (atomic append)
- **Config:** Loaded once at startup (no hot-reload)

## 9. Known Constraints

1. **Requires `gh` CLI** for PR creation (not bundled)
2. **Requires `git`** in PATH (not using git2 crate for simplicity)
3. **SSH for Docker checks** requires key-based auth (no password prompt)
4. **AI review (Phase 3)** requires OpenRouter API key
5. **MCP server** stdio only (no HTTP transport)
6. **Single project per invocation** (run from project root)
7. **No Windows-specific path handling** yet (uses Unix paths in config)
8. **`ship note` graceful degradation:** vault missing / write fail → warning on stderr, exit 0 (never fails ship).
9. **`ship note` auto-log hook is CLI-level only:** applies to `ship check` command invocation; MCP `ship_check` does NOT auto-trigger (callers use `ship_note_export` explicitly). Intentional — prevents unexpected vault writes from AI agents.
10. **`ship note` diacritic stripping:** manual Vietnamese table (no `unicode-normalization` dep). Other non-ASCII scripts (Chinese/Japanese/Arabic) fall through as `-` in slugify. Acceptable for the current single-user scope.
