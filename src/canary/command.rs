use crate::canary::{CanaryStatus, HealthCheck};
use crate::config::CustomCheckConfig;
use std::process::Command;
use std::time::Instant;

/// Run a custom command check, optionally via SSH.
pub fn check(custom: &CustomCheckConfig, ssh: Option<&str>) -> HealthCheck {
    let start = Instant::now();

    let output = if let Some(ssh_str) = ssh {
        run_via_ssh(ssh_str, &custom.command)
    } else {
        run_local(&custom.command)
    };

    let latency = start.elapsed().as_millis() as u64;

    match output {
        Ok(stdout) => HealthCheck {
            name: custom.name.clone(),
            status: CanaryStatus::Healthy,
            latency_ms: Some(latency),
            details: if stdout.is_empty() {
                None
            } else {
                Some(stdout)
            },
        },
        Err(msg) => HealthCheck {
            name: custom.name.clone(),
            status: CanaryStatus::Down(msg),
            latency_ms: Some(latency),
            details: None,
        },
    }
}

fn run_local(cmd: &str) -> std::result::Result<String, String> {
    let output = Command::new("sh")
        .args(["-c", cmd])
        .output()
        .map_err(|e| format!("failed to execute: {e}"))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(if stderr.is_empty() {
            format!("exit code {}", output.status.code().unwrap_or(-1))
        } else {
            stderr
        })
    }
}

fn run_via_ssh(ssh: &str, cmd: &str) -> std::result::Result<String, String> {
    let (ssh_target, port) = parse_ssh(ssh);

    let mut ssh_cmd = Command::new("ssh");
    ssh_cmd
        .args(["-o", "StrictHostKeyChecking=no"])
        .args(["-o", "ConnectTimeout=10"])
        .args(["-o", "BatchMode=yes"]);

    if let Some(p) = &port {
        ssh_cmd.args(["-p", p]);
    }

    ssh_cmd.arg(&ssh_target).arg(cmd);

    let output = ssh_cmd
        .output()
        .map_err(|e| format!("ssh failed: {e}"))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(if stderr.contains("Permission denied") {
            "SSH auth failed".into()
        } else if stderr.contains("Connection refused") || stderr.contains("timed out") {
            "SSH connection failed".into()
        } else if stderr.is_empty() {
            format!("exit code {}", output.status.code().unwrap_or(-1))
        } else {
            stderr
        })
    }
}

fn parse_ssh(ssh: &str) -> (String, Option<String>) {
    if let Some((target, port)) = ssh.rsplit_once(':')
        && port.parse::<u16>().is_ok()
    {
        return (target.to_string(), Some(port.to_string()));
    }
    (ssh.to_string(), None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_local_success() {
        let result = run_local("echo hello");
        assert_eq!(result.unwrap(), "hello");
    }

    #[test]
    fn test_run_local_failure() {
        let result = run_local("false");
        assert!(result.is_err());
    }

    #[test]
    fn test_check_local_healthy() {
        let custom = CustomCheckConfig {
            name: "Test".into(),
            command: "echo ok".into(),
        };
        let result = check(&custom, None);
        assert!(result.status.is_healthy());
        assert_eq!(result.name, "Test");
    }

    #[test]
    fn test_check_local_down() {
        let custom = CustomCheckConfig {
            name: "Fail".into(),
            command: "false".into(),
        };
        let result = check(&custom, None);
        assert!(matches!(result.status, CanaryStatus::Down(_)));
    }
}
