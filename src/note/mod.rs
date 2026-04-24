use crate::config::ObsidianConfig;
use chrono::Local;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Default)]
pub struct NoteOptions {
    pub project: Option<String>,
    pub ticket: Option<String>,
    pub message: Option<String>,
    pub vault_path: Option<String>,
}

#[derive(Debug)]
pub enum NoteOutcome {
    Written(PathBuf),
    Skipped(String),
}

/// Export a ship note to the Obsidian vault. Graceful: any vault-side
/// failure is returned as `NoteOutcome::Skipped(reason)` and never errors.
pub fn run(config: &ObsidianConfig, opts: NoteOptions) -> NoteOutcome {
    match write_note(config, opts) {
        Ok(path) => NoteOutcome::Written(path),
        Err(reason) => NoteOutcome::Skipped(reason),
    }
}

fn write_note(config: &ObsidianConfig, opts: NoteOptions) -> Result<PathBuf, String> {
    let project = resolve_project(config, opts.project.as_deref());
    let vault = resolve_vault(config, opts.vault_path.as_deref())?;

    let logs_dir = vault.join("10_Projects").join(&project).join("logs");
    std::fs::create_dir_all(&logs_dir)
        .map_err(|e| format!("cannot create {}: {e}", logs_dir.display()))?;

    let info = collect_git_info();
    let today = Local::now().format("%Y-%m-%d").to_string();

    let slug = {
        let s = slugify(&info.commit_subject);
        if s.is_empty() { "note".to_string() } else { s }
    };

    let target = pick_filename(&logs_dir, &today, &slug);
    let content = build_content(
        &project,
        &today,
        opts.ticket.as_deref(),
        opts.message.as_deref(),
        &info,
    );

    atomic_write(&target, &content).map_err(|e| format!("write failed: {e}"))?;
    Ok(target)
}

fn resolve_project(config: &ObsidianConfig, arg: Option<&str>) -> String {
    arg.map(String::from)
        .or_else(|| config.project_slug.clone())
        .or_else(|| {
            std::env::current_dir()
                .ok()
                .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        })
        .unwrap_or_else(|| "unknown".into())
}

fn resolve_vault(config: &ObsidianConfig, arg: Option<&str>) -> Result<PathBuf, String> {
    let raw = arg
        .map(String::from)
        .or_else(|| std::env::var("OBSIDIAN_VAULT_PATH").ok())
        .or_else(|| config.vault_path.clone())
        .unwrap_or_else(|| "~/VibeNotes".into());

    let path = PathBuf::from(shellexpand(&raw));
    if !path.exists() {
        return Err(format!("vault path does not exist: {}", path.display()));
    }
    if !path.is_dir() {
        return Err(format!("vault path is not a directory: {}", path.display()));
    }
    Ok(path)
}

fn pick_filename(logs_dir: &Path, today: &str, slug: &str) -> PathBuf {
    let target = logs_dir.join(format!("{today}-{slug}.md"));
    if !target.exists() {
        return target;
    }
    let suffix = rand_hex4();
    logs_dir.join(format!("{today}-{slug}-{suffix}.md"))
}

fn atomic_write(target: &Path, content: &str) -> std::io::Result<()> {
    let tmp = target.with_extension(format!("tmp.{}", rand_hex4()));
    std::fs::write(&tmp, content)?;
    std::fs::rename(&tmp, target)?;
    Ok(())
}

fn rand_hex4() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    format!("{:04x}", nanos & 0xFFFF)
}

// ---------- Content ----------

fn build_content(
    project: &str,
    date: &str,
    ticket: Option<&str>,
    message: Option<&str>,
    info: &GitInfo,
) -> String {
    let ticket_line = ticket.unwrap_or("");
    let mut out = format!(
        "---\ndate: {date}\nproject: {project}\nticket: {ticket_line}\ntype: ship-note\ntags: [project-log, ship]\n---\n\n# {project} — {date}\n\n"
    );

    if !info.commit_subject.is_empty() {
        out.push_str("## Changes\n");
        out.push_str(&info.commit_subject);
        out.push_str("\n\n");
        if let Some(body) = &info.commit_body {
            out.push_str(body);
            out.push_str("\n\n");
        }
    }

    if let Some(stat) = info.diff_stat.as_deref().filter(|s| !s.is_empty()) {
        out.push_str("## Files changed\n```\n");
        for line in stat.lines().take(20) {
            out.push_str(line);
            out.push('\n');
        }
        out.push_str("```\n\n");
    }

    if let Some(msg) = message {
        out.push_str("## Learnings\n");
        out.push_str(msg);
        out.push_str("\n\n");
    }

    out.push_str("## Related\n");
    if let Some(hash) = &info.commit_hash {
        if let Some(url) = &info.github_commit_url {
            out.push_str(&format!("- Commit: {hash} — {url}\n"));
        } else {
            out.push_str(&format!("- Commit: {hash}\n"));
        }
    }
    if let Some(branch) = &info.branch {
        out.push_str(&format!("- Branch: {branch}\n"));
    }
    if let Some(pr) = &info.pr_url {
        out.push_str(&format!("- PR: {pr}\n"));
    }

    out
}

// ---------- Git info ----------

#[derive(Debug, Default)]
struct GitInfo {
    commit_hash: Option<String>,
    commit_subject: String,
    commit_body: Option<String>,
    branch: Option<String>,
    diff_stat: Option<String>,
    github_commit_url: Option<String>,
    pr_url: Option<String>,
}

fn collect_git_info() -> GitInfo {
    let commit_hash = git_cmd(&["rev-parse", "--short=7", "HEAD"]);
    let commit_subject = git_cmd(&["log", "-1", "--format=%s"]).unwrap_or_default();
    let commit_body = git_cmd(&["log", "-1", "--format=%b"]).filter(|s| !s.trim().is_empty());
    let branch = git_cmd(&["rev-parse", "--abbrev-ref", "HEAD"]);
    let diff_stat = git_cmd(&["diff", "--stat", "HEAD~1..HEAD"]);
    let github_commit_url = match (&github_repo_url(), &commit_hash) {
        (Some(base), Some(hash)) => Some(format!("{base}/commit/{hash}")),
        _ => None,
    };
    let pr_url = gh_pr_url();

    GitInfo {
        commit_hash,
        commit_subject,
        commit_body,
        branch,
        diff_stat,
        github_commit_url,
        pr_url,
    }
}

fn git_cmd(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

fn github_repo_url() -> Option<String> {
    let raw = git_cmd(&["remote", "get-url", "origin"])?;
    if let Some(rest) = raw.strip_prefix("git@github.com:") {
        let clean = rest.trim_end_matches(".git");
        return Some(format!("https://github.com/{clean}"));
    }
    if raw.contains("github.com") {
        return Some(
            raw.trim_end_matches(".git")
                .trim_end_matches('/')
                .to_string(),
        );
    }
    None
}

fn gh_pr_url() -> Option<String> {
    let out = Command::new("gh")
        .args(["pr", "view", "--json", "url", "-q", ".url"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

// ---------- Slug ----------

fn slugify(subject: &str) -> String {
    let stripped = strip_diacritics_vi(subject);
    let kebab = kebab_case(&stripped);
    truncate_kebab(&kebab, 40)
}

fn truncate_kebab(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        return s.to_string();
    }
    // Safe truncation on char boundary, then strip trailing dashes
    let mut trunc: String = s
        .chars()
        .scan(0usize, |acc, c| {
            let next = *acc + c.len_utf8();
            if next > max_len {
                None
            } else {
                *acc = next;
                Some(c)
            }
        })
        .collect();
    while trunc.ends_with('-') {
        trunc.pop();
    }
    trunc
}

fn kebab_case(s: &str) -> String {
    let lower = s.to_lowercase();
    let mut out = String::with_capacity(lower.len());
    let mut prev_dash = true; // treat leading as dash so leading junk doesn't produce '-'
    for c in lower.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

fn strip_diacritics_vi(s: &str) -> String {
    s.chars().map(map_vi_char).collect()
}

fn map_vi_char(c: char) -> char {
    match c {
        'á' | 'à' | 'ả' | 'ã' | 'ạ' | 'ă' | 'ắ' | 'ằ' | 'ẳ' | 'ẵ' | 'ặ' | 'â' | 'ấ' | 'ầ' | 'ẩ'
        | 'ẫ' | 'ậ' => 'a',
        'Á' | 'À' | 'Ả' | 'Ã' | 'Ạ' | 'Ă' | 'Ắ' | 'Ằ' | 'Ẳ' | 'Ẵ' | 'Ặ' | 'Â' | 'Ấ' | 'Ầ' | 'Ẩ'
        | 'Ẫ' | 'Ậ' => 'A',
        'é' | 'è' | 'ẻ' | 'ẽ' | 'ẹ' | 'ê' | 'ế' | 'ề' | 'ể' | 'ễ' | 'ệ' => 'e',
        'É' | 'È' | 'Ẻ' | 'Ẽ' | 'Ẹ' | 'Ê' | 'Ế' | 'Ề' | 'Ể' | 'Ễ' | 'Ệ' => 'E',
        'í' | 'ì' | 'ỉ' | 'ĩ' | 'ị' => 'i',
        'Í' | 'Ì' | 'Ỉ' | 'Ĩ' | 'Ị' => 'I',
        'ó' | 'ò' | 'ỏ' | 'õ' | 'ọ' | 'ô' | 'ố' | 'ồ' | 'ổ' | 'ỗ' | 'ộ' | 'ơ' | 'ớ' | 'ờ' | 'ở'
        | 'ỡ' | 'ợ' => 'o',
        'Ó' | 'Ò' | 'Ỏ' | 'Õ' | 'Ọ' | 'Ô' | 'Ố' | 'Ồ' | 'Ổ' | 'Ỗ' | 'Ộ' | 'Ơ' | 'Ớ' | 'Ờ' | 'Ở'
        | 'Ỡ' | 'Ợ' => 'O',
        'ú' | 'ù' | 'ủ' | 'ũ' | 'ụ' | 'ư' | 'ứ' | 'ừ' | 'ử' | 'ữ' | 'ự' => 'u',
        'Ú' | 'Ù' | 'Ủ' | 'Ũ' | 'Ụ' | 'Ư' | 'Ứ' | 'Ừ' | 'Ử' | 'Ữ' | 'Ự' => 'U',
        'ý' | 'ỳ' | 'ỷ' | 'ỹ' | 'ỵ' => 'y',
        'Ý' | 'Ỳ' | 'Ỷ' | 'Ỹ' | 'Ỵ' => 'Y',
        'đ' => 'd',
        'Đ' => 'D',
        _ => c,
    }
}

// ---------- Shell expand (zero-dep, matches pattern in crate::learn) ----------

fn shellexpand(path: &str) -> String {
    shellexpand_with_home(path, home_dir().as_deref())
}

fn shellexpand_with_home(path: &str, home: Option<&str>) -> String {
    if let Some(rest) = path.strip_prefix("~/")
        && let Some(h) = home
    {
        return format!("{h}/{rest}");
    }
    path.to_string()
}

fn home_dir() -> Option<String> {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_diacritics_vi() {
        assert_eq!(
            strip_diacritics_vi("Đây là tiếng Việt"),
            "Day la tieng Viet"
        );
        assert_eq!(
            strip_diacritics_vi("ăn quả nhớ kẻ trồng cây"),
            "an qua nho ke trong cay"
        );
        assert_eq!(strip_diacritics_vi("plain ASCII"), "plain ASCII");
    }

    #[test]
    fn test_kebab_case() {
        assert_eq!(kebab_case("Hello World"), "hello-world");
        assert_eq!(kebab_case("foo  bar___baz"), "foo-bar-baz");
        assert_eq!(kebab_case("already-kebab"), "already-kebab");
        assert_eq!(kebab_case("   leading trailing   "), "leading-trailing");
        assert_eq!(kebab_case(""), "");
    }

    #[test]
    fn test_slugify_vietnamese() {
        assert_eq!(
            slugify("Thêm tính năng đặc biệt"),
            "them-tinh-nang-dac-biet"
        );
        assert_eq!(
            slugify("feat: add retry logic to HTTP canary"),
            "feat-add-retry-logic-to-http-canary"
        );
    }

    #[test]
    fn test_slugify_truncation() {
        let long = "a".repeat(60);
        let out = slugify(&long);
        assert!(out.len() <= 40);
        assert_eq!(out, "a".repeat(40));
    }

    #[test]
    fn test_truncate_strips_trailing_dash() {
        assert_eq!(truncate_kebab("hello-world-foo", 12), "hello-world");
    }

    #[test]
    fn test_shellexpand_tilde_with_home() {
        assert_eq!(
            shellexpand_with_home("~/foo/bar", Some("/tmp/testhome")),
            "/tmp/testhome/foo/bar"
        );
        assert_eq!(
            shellexpand_with_home("/abs/path", Some("/tmp/testhome")),
            "/abs/path"
        );
        assert_eq!(
            shellexpand_with_home("relative/path", Some("/tmp/testhome")),
            "relative/path"
        );
        assert_eq!(shellexpand_with_home("~/foo", None), "~/foo");
    }

    #[test]
    fn test_resolve_project_arg_wins() {
        let cfg = ObsidianConfig {
            project_slug: Some("config-slug".into()),
            ..Default::default()
        };
        assert_eq!(resolve_project(&cfg, Some("arg-slug")), "arg-slug");
    }

    #[test]
    fn test_resolve_project_config_fallback() {
        let cfg = ObsidianConfig {
            project_slug: Some("config-slug".into()),
            ..Default::default()
        };
        assert_eq!(resolve_project(&cfg, None), "config-slug");
    }

    #[test]
    fn test_resolve_vault_missing() {
        let cfg = ObsidianConfig::default();
        let err = resolve_vault(&cfg, Some("/nonexistent/vault/path/xyz")).unwrap_err();
        assert!(err.contains("does not exist"));
    }

    #[test]
    fn test_resolve_vault_not_dir() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let cfg = ObsidianConfig::default();
        let err = resolve_vault(&cfg, Some(tmp.path().to_str().unwrap())).unwrap_err();
        assert!(err.contains("not a directory"));
    }

    #[test]
    fn test_atomic_write_creates_file() {
        let dir = tempfile::TempDir::new().unwrap();
        let target = dir.path().join("test.md");
        atomic_write(&target, "hello").unwrap();
        assert_eq!(std::fs::read_to_string(&target).unwrap(), "hello");
    }

    #[test]
    fn test_pick_filename_collision() {
        let dir = tempfile::TempDir::new().unwrap();
        let first = pick_filename(dir.path(), "2026-04-24", "foo");
        std::fs::write(&first, "x").unwrap();
        let second = pick_filename(dir.path(), "2026-04-24", "foo");
        assert_ne!(first, second);
        assert!(
            second
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("2026-04-24-foo-")
        );
    }

    #[test]
    fn test_run_vault_missing_returns_skipped() {
        let cfg = ObsidianConfig {
            vault_path: Some("/definitely/not/a/vault".into()),
            ..Default::default()
        };
        let outcome = run(&cfg, NoteOptions::default());
        match outcome {
            NoteOutcome::Skipped(r) => assert!(r.contains("does not exist")),
            _ => panic!("expected Skipped"),
        }
    }

    #[test]
    fn test_build_content_includes_learnings_when_message_present() {
        let info = GitInfo {
            commit_subject: "test commit".into(),
            ..Default::default()
        };
        let content = build_content("foo", "2026-04-24", None, Some("a learning"), &info);
        assert!(content.contains("## Learnings\na learning"));
    }

    #[test]
    fn test_build_content_omits_learnings_when_no_message() {
        let info = GitInfo {
            commit_subject: "test commit".into(),
            ..Default::default()
        };
        let content = build_content("foo", "2026-04-24", None, None, &info);
        assert!(!content.contains("## Learnings"));
    }

    #[test]
    fn test_run_writes_file_when_vault_exists() {
        let dir = tempfile::TempDir::new().unwrap();
        let cfg = ObsidianConfig {
            vault_path: Some(dir.path().to_string_lossy().into_owned()),
            ..Default::default()
        };
        let opts = NoteOptions {
            project: Some("testproj".into()),
            message: Some("hi".into()),
            ..Default::default()
        };
        let outcome = run(&cfg, opts);
        match outcome {
            NoteOutcome::Written(path) => {
                assert!(path.exists());
                assert!(
                    path.to_string_lossy()
                        .contains("10_Projects/testproj/logs/")
                );
                let body = std::fs::read_to_string(&path).unwrap();
                assert!(body.contains("project: testproj"));
                assert!(body.contains("## Learnings\nhi"));
            }
            NoteOutcome::Skipped(r) => panic!("expected Written, got Skipped: {r}"),
        }
    }
}
