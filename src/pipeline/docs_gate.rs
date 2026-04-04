use crate::config::Config;
use crate::error::Result;
use crate::pipeline::{StepResult, StepStatus};
use std::process::Command;
use std::time::Instant;

pub fn run(config: &Config) -> Result<StepResult> {
    let start = Instant::now();

    // Check if docs-gate binary exists
    let which = Command::new("which").arg("docs-gate").output();
    let exists = which.map(|o| o.status.success()).unwrap_or(false);

    if !exists {
        // Also check where on Windows
        let where_cmd = Command::new("where").arg("docs-gate").output();
        let exists_win = where_cmd.map(|o| o.status.success()).unwrap_or(false);

        if !exists_win {
            return Ok(StepResult {
                name: "Docs Gate".into(),
                status: StepStatus::Skip("docs-gate not found in PATH".into()),
                duration: start.elapsed(),
                output: None,
            });
        }
    }

    // Check if .docs-gate.toml exists (optional but informative)
    let has_config = std::path::Path::new(".docs-gate.toml").exists();

    let output = Command::new("docs-gate").arg("--all").output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    if output.status.success() {
        Ok(StepResult {
            name: "Docs Gate".into(),
            status: StepStatus::Pass,
            duration: start.elapsed(),
            output: Some(format!(
                "all checks passed{}",
                if has_config { "" } else { " (default config)" }
            )),
        })
    } else {
        let status = if config.docs_gate.blocking {
            StepStatus::Fail(format!("docs-gate failed:\n{stdout}"))
        } else {
            StepStatus::Warn(format!("docs-gate issues:\n{stdout}"))
        };

        Ok(StepResult {
            name: "Docs Gate".into(),
            status,
            duration: start.elapsed(),
            output: None,
        })
    }
}
