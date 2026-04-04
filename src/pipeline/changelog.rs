use crate::config::Config;
use crate::error::Result;
use crate::pipeline::{StepResult, StepStatus};
use regex::Regex;
use std::process::Command;
use std::time::Instant;

pub fn run(config: &Config) -> Result<StepResult> {
    let start = Instant::now();

    let commits = get_branch_commits(&config.base_branch)?;
    if commits.is_empty() {
        return Ok(StepResult {
            name: "Changelog".into(),
            status: StepStatus::Skip("no commits to log".into()),
            duration: start.elapsed(),
            output: None,
        });
    }

    let version = read_current_version();
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    let entry = generate_entry(&version, &date, &commits);

    let changelog_path = &config.changelog.file;
    prepend_entry(changelog_path, &entry)?;

    Ok(StepResult {
        name: "Changelog".into(),
        status: StepStatus::Pass,
        duration: start.elapsed(),
        output: Some(format!("{} commits → {}", commits.len(), changelog_path)),
    })
}

fn get_branch_commits(base: &str) -> Result<Vec<CommitInfo>> {
    let output = Command::new("git")
        .args([
            "log",
            &format!("{base}..HEAD"),
            "--pretty=format:%s",
            "--reverse",
        ])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let commits = stdout
        .lines()
        .filter(|l| !l.is_empty())
        .map(|line| parse_commit(line))
        .collect();

    Ok(commits)
}

#[derive(Debug)]
struct CommitInfo {
    kind: String,
    message: String,
}

fn parse_commit(line: &str) -> CommitInfo {
    let re = Regex::new(r"^(feat|fix|refactor|docs|test|chore|perf|style|ci):\s*(.+)$").unwrap();

    if let Some(caps) = re.captures(line) {
        CommitInfo {
            kind: caps[1].to_string(),
            message: caps[2].to_string(),
        }
    } else {
        CommitInfo {
            kind: "other".into(),
            message: line.to_string(),
        }
    }
}

fn generate_entry(version: &str, date: &str, commits: &[CommitInfo]) -> String {
    let mut sections: Vec<(&str, Vec<&str>)> = Vec::new();

    let mut added = Vec::new();
    let mut fixed = Vec::new();
    let mut improved = Vec::new();
    let mut docs = Vec::new();
    let mut other = Vec::new();

    for c in commits {
        match c.kind.as_str() {
            "feat" => added.push(c.message.as_str()),
            "fix" => fixed.push(c.message.as_str()),
            "refactor" | "perf" => improved.push(c.message.as_str()),
            "docs" => docs.push(c.message.as_str()),
            "chore" | "ci" | "style" | "test" => {} // Skip in changelog
            _ => other.push(c.message.as_str()),
        }
    }

    if !added.is_empty() {
        sections.push(("Added", added));
    }
    if !fixed.is_empty() {
        sections.push(("Fixed", fixed));
    }
    if !improved.is_empty() {
        sections.push(("Improved", improved));
    }
    if !docs.is_empty() {
        sections.push(("Documentation", docs));
    }
    if !other.is_empty() {
        sections.push(("Other", other));
    }

    let mut entry = format!("## {version} ({date})\n");
    for (heading, items) in sections {
        entry.push_str(&format!("\n### {heading}\n"));
        for item in items {
            entry.push_str(&format!("- {item}\n"));
        }
    }

    entry
}

fn prepend_entry(path: &str, entry: &str) -> Result<()> {
    let existing = std::fs::read_to_string(path).unwrap_or_else(|_| "# Changelog\n".into());

    // Insert after the first heading line
    let content = if let Some(pos) = existing.find("\n## ") {
        format!("{}\n{}{}", &existing[..pos], entry, &existing[pos..])
    } else {
        format!("{}\n{}", existing.trim(), entry)
    };

    std::fs::write(path, content)?;
    Ok(())
}

fn read_current_version() -> String {
    for path in ["VERSION", "version.txt"] {
        if let Ok(v) = std::fs::read_to_string(path) {
            let trimmed = v.trim().to_string();
            if !trimmed.is_empty() {
                return trimmed;
            }
        }
    }
    "0.1.0".into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_conventional_commit() {
        let c = parse_commit("feat: add health check endpoint");
        assert_eq!(c.kind, "feat");
        assert_eq!(c.message, "add health check endpoint");
    }

    #[test]
    fn test_parse_non_conventional() {
        let c = parse_commit("random commit message");
        assert_eq!(c.kind, "other");
        assert_eq!(c.message, "random commit message");
    }

    #[test]
    fn test_generate_entry() {
        let commits = vec![
            CommitInfo {
                kind: "feat".into(),
                message: "add ship command".into(),
            },
            CommitInfo {
                kind: "fix".into(),
                message: "handle empty diff".into(),
            },
        ];
        let entry = generate_entry("0.2.0", "2026-04-04", &commits);
        assert!(entry.contains("## 0.2.0 (2026-04-04)"));
        assert!(entry.contains("### Added"));
        assert!(entry.contains("- add ship command"));
        assert!(entry.contains("### Fixed"));
        assert!(entry.contains("- handle empty diff"));
    }
}
