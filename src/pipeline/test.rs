use crate::config::Config;
use crate::detect::ProjectStack;
use crate::error::Result;
use crate::pipeline::{StepResult, StepStatus};
use std::process::Command;
use std::time::Instant;

pub fn run(config: &Config) -> Result<StepResult> {
    let start = Instant::now();

    let test_cmd = resolve_test_command(config);
    let Some(cmd) = test_cmd else {
        return Ok(StepResult {
            name: "Test".into(),
            status: StepStatus::Warn("No test command detected".into()),
            duration: start.elapsed(),
            output: None,
        });
    };

    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let (program, args) = parts.split_first().expect("test command not empty");

    let output = Command::new(program).args(args).output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        Ok(StepResult {
            name: "Test".into(),
            status: StepStatus::Pass,
            duration: start.elapsed(),
            output: Some(format!("passed ({})", cmd)),
        })
    } else {
        let detail = if !stderr.is_empty() {
            // Take last 5 lines of stderr for context
            stderr.lines().rev().take(5).collect::<Vec<_>>().join("\n")
        } else {
            stdout
                .lines()
                .rev()
                .take(5)
                .collect::<Vec<_>>()
                .join("\n")
        };

        Ok(StepResult {
            name: "Test".into(),
            status: StepStatus::Fail(format!("tests failed ({})\n{}", cmd, detail)),
            duration: start.elapsed(),
            output: None,
        })
    }
}

fn resolve_test_command(config: &Config) -> Option<String> {
    // Config override first
    if let Some(cmd) = &config.test.command {
        return Some(cmd.clone());
    }

    // Auto-detect from stack
    let cwd = std::env::current_dir().ok()?;
    let stack = ProjectStack::detect(&cwd);
    stack.test_command().map(String::from)
}
