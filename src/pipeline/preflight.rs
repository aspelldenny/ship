use crate::config::Config;
use crate::error::{Result, ShipError};
use crate::pipeline::{StepResult, StepStatus};
use std::process::Command;
use std::time::Instant;

pub fn run(config: &Config) -> Result<StepResult> {
    let start = Instant::now();

    // Check current branch
    let branch = current_branch()?;
    let protected = [&config.base_branch, "master", "main"];
    if protected.contains(&branch.as_str()) {
        return Ok(StepResult {
            name: "Preflight".into(),
            status: StepStatus::Fail(format!("On protected branch: {branch}")),
            duration: start.elapsed(),
            output: None,
        });
    }

    // Check git status
    let status = git_status()?;
    let diff_stats = diff_stats(&config.base_branch)?;

    let detail = format!(
        "branch: {branch}, {diff_stats}{}",
        if status.is_empty() {
            String::new()
        } else {
            format!(", {} uncommitted", status.lines().count())
        }
    );

    Ok(StepResult {
        name: "Preflight".into(),
        status: StepStatus::Pass,
        duration: start.elapsed(),
        output: Some(detail),
    })
}

fn current_branch() -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()?;

    if !output.status.success() {
        return Err(ShipError::Git("Not a git repository".into()));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn git_status() -> Result<String> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()?;

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn diff_stats(base: &str) -> Result<String> {
    let output = Command::new("git")
        .args(["diff", "--stat", &format!("{base}...HEAD")])
        .output()?;

    if output.status.success() {
        let stat = String::from_utf8_lossy(&output.stdout);
        let last_line = stat.lines().last().unwrap_or("no changes");
        Ok(last_line.trim().to_string())
    } else {
        Ok("diff unavailable".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_branch_in_git_repo() {
        // This test runs in the ship repo itself
        let branch = current_branch();
        // Should succeed or fail gracefully
        assert!(branch.is_ok() || branch.is_err());
    }
}
