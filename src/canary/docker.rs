use crate::canary::{CanaryStatus, HealthCheck};
use std::process::Command;
use std::time::Instant;

pub fn check(container: &str, ssh: &str) -> HealthCheck {
    let start = Instant::now();

    // Parse ssh string: "user@host:port" or "user@host"
    let (ssh_target, port) = parse_ssh(ssh);

    // Build SSH command to check docker container
    let docker_cmd = format!("docker inspect --format='{{{{.State.Status}}}}' {container}");

    let mut cmd = Command::new("ssh");
    cmd.args(["-o", "StrictHostKeyChecking=no"])
        .args(["-o", "ConnectTimeout=10"])
        .args(["-o", "BatchMode=yes"]);

    if let Some(p) = port {
        cmd.args(["-p", &p]);
    }

    cmd.arg(&ssh_target).arg(&docker_cmd);

    match cmd.output() {
        Ok(output) => {
            let latency = start.elapsed().as_millis() as u64;
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

            if !output.status.success() {
                let msg = if stderr.contains("Permission denied") {
                    "SSH auth failed".into()
                } else if stderr.contains("Connection refused") || stderr.contains("timed out") {
                    "SSH connection failed".into()
                } else if stderr.contains("No such object") {
                    format!("container '{container}' not found")
                } else {
                    format!("SSH error: {stderr}")
                };

                return HealthCheck {
                    name: format!("Docker {container}"),
                    status: CanaryStatus::Down(msg),
                    latency_ms: Some(latency),
                    details: if !stderr.is_empty() {
                        Some(stderr)
                    } else {
                        None
                    },
                };
            }

            // Parse container status
            let status_str = stdout.trim_matches('\'').to_lowercase();
            match status_str.as_str() {
                "running" => HealthCheck {
                    name: format!("Docker {container}"),
                    status: CanaryStatus::Healthy,
                    latency_ms: Some(latency),
                    details: Some(format!("status: {status_str}")),
                },
                "restarting" => HealthCheck {
                    name: format!("Docker {container}"),
                    status: CanaryStatus::Degraded("container restarting".into()),
                    latency_ms: Some(latency),
                    details: Some(format!("status: {status_str}")),
                },
                _ => HealthCheck {
                    name: format!("Docker {container}"),
                    status: CanaryStatus::Down(format!("container status: {status_str}")),
                    latency_ms: Some(latency),
                    details: Some(format!("status: {status_str}")),
                },
            }
        }
        Err(e) => HealthCheck {
            name: format!("Docker {container}"),
            status: CanaryStatus::Down(format!("ssh command failed: {e}")),
            latency_ms: None,
            details: None,
        },
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
    fn test_parse_ssh_with_port() {
        let (target, port) = parse_ssh("deploy@103.167.150.178:1994");
        assert_eq!(target, "deploy@103.167.150.178");
        assert_eq!(port, Some("1994".into()));
    }

    #[test]
    fn test_parse_ssh_without_port() {
        let (target, port) = parse_ssh("deploy@myserver.com");
        assert_eq!(target, "deploy@myserver.com");
        assert_eq!(port, None);
    }
}
