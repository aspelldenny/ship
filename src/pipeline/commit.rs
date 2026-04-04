use crate::config::Config;
use crate::error::Result;
use crate::pipeline::{StepResult, StepStatus};
use std::process::Command;
use std::time::Instant;

pub fn run(config: &Config) -> Result<StepResult> {
    let start = Instant::now();

    // Stage all changes
    let stage = Command::new("git").args(["add", "-A"]).output()?;

    if !stage.status.success() {
        return Ok(StepResult {
            name: "Commit".into(),
            status: StepStatus::Fail("git add failed".into()),
            duration: start.elapsed(),
            output: None,
        });
    }

    // Check if there's anything to commit
    let status = Command::new("git")
        .args(["status", "--porcelain"])
        .output()?;
    let status_str = String::from_utf8_lossy(&status.stdout);
    if status_str.trim().is_empty() {
        return Ok(StepResult {
            name: "Commit".into(),
            status: StepStatus::Skip("nothing to commit".into()),
            duration: start.elapsed(),
            output: None,
        });
    }

    // Read VERSION for commit message
    let version = read_version();
    let project = config.project_name();

    let message = format!(
        "chore: ship {project} v{version}\n\n\
         Automated release via ship CLI.\n\n\
         Co-Authored-By: ship-cli <noreply@ship.dev>"
    );

    let commit = Command::new("git")
        .args(["commit", "-m", &message])
        .output()?;

    if commit.status.success() {
        Ok(StepResult {
            name: "Commit".into(),
            status: StepStatus::Pass,
            duration: start.elapsed(),
            output: Some(format!("v{version}")),
        })
    } else {
        let stderr = String::from_utf8_lossy(&commit.stderr);
        Ok(StepResult {
            name: "Commit".into(),
            status: StepStatus::Fail(format!("commit failed: {stderr}")),
            duration: start.elapsed(),
            output: None,
        })
    }
}

fn read_version() -> String {
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
