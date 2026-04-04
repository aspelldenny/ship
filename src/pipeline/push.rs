use crate::error::Result;
use crate::pipeline::{StepResult, StepStatus};
use std::process::Command;
use std::time::Instant;

pub fn run() -> Result<StepResult> {
    let start = Instant::now();

    let branch = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()?;
    let branch_name = String::from_utf8_lossy(&branch.stdout).trim().to_string();

    let push = Command::new("git")
        .args(["push", "-u", "origin", &branch_name])
        .output()?;

    if push.status.success() {
        Ok(StepResult {
            name: "Push".into(),
            status: StepStatus::Pass,
            duration: start.elapsed(),
            output: Some(format!("��� origin/{branch_name}")),
        })
    } else {
        let stderr = String::from_utf8_lossy(&push.stderr);
        Ok(StepResult {
            name: "Push".into(),
            status: StepStatus::Fail(format!("push failed: {stderr}")),
            duration: start.elapsed(),
            output: None,
        })
    }
}
