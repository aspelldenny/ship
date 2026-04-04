pub mod docker;
pub mod http;

use crate::config::CanaryConfig;
use crate::error::Result;
use crate::output;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct HealthCheck {
    pub name: String,
    pub status: CanaryStatus,
    pub latency_ms: Option<u64>,
    pub details: Option<String>,
}

#[derive(Debug, Clone)]
pub enum CanaryStatus {
    Healthy,
    Degraded(String),
    Down(String),
}

impl CanaryStatus {
    pub fn is_healthy(&self) -> bool {
        matches!(self, Self::Healthy)
    }
}

pub struct CanaryResult {
    pub checks: Vec<HealthCheck>,
    pub duration: Duration,
}

impl CanaryResult {
    pub fn all_healthy(&self) -> bool {
        self.checks.iter().all(|c| c.status.is_healthy())
    }

    pub fn any_down(&self) -> bool {
        self.checks
            .iter()
            .any(|c| matches!(c.status, CanaryStatus::Down(_)))
    }
}

pub async fn run(config: &CanaryConfig) -> Result<CanaryResult> {
    let start = Instant::now();
    let mut checks: Vec<HealthCheck> = Vec::new();

    output::header("Canary Health Check");

    for check_type in &config.checks {
        let result = match check_type.as_str() {
            "http" => {
                if let Some(url) = &config.url {
                    http::check(url, config.timeout_secs).await
                } else {
                    HealthCheck {
                        name: "HTTP".into(),
                        status: CanaryStatus::Down("no URL configured".into()),
                        latency_ms: None,
                        details: None,
                    }
                }
            }
            "docker" => {
                if let (Some(container), Some(ssh)) = (&config.docker_container, &config.ssh) {
                    docker::check(container, ssh)
                } else {
                    HealthCheck {
                        name: "Docker".into(),
                        status: CanaryStatus::Down("docker_container or ssh not configured".into()),
                        latency_ms: None,
                        details: None,
                    }
                }
            }
            other => HealthCheck {
                name: other.to_string(),
                status: CanaryStatus::Down(format!("unknown check type: {other}")),
                latency_ms: None,
                details: None,
            },
        };

        print_check(&result);
        checks.push(result);
    }

    let result = CanaryResult {
        checks,
        duration: start.elapsed(),
    };

    print_summary(&result);
    Ok(result)
}

fn print_check(check: &HealthCheck) {
    let latency = check
        .latency_ms
        .map(|ms| format!(" ({ms}ms)"))
        .unwrap_or_default();

    match &check.status {
        CanaryStatus::Healthy => {
            eprintln!("  ✅ {}{latency}", check.name);
        }
        CanaryStatus::Degraded(msg) => {
            eprintln!("  ⚠️  {} — {msg}{latency}", check.name);
        }
        CanaryStatus::Down(msg) => {
            eprintln!("  ❌ {} — {msg}", check.name);
        }
    }

    if let Some(details) = &check.details {
        for line in details.lines() {
            eprintln!("     {line}");
        }
    }
}

fn print_summary(result: &CanaryResult) {
    eprintln!("{}", "─".repeat(50));
    if result.all_healthy() {
        eprintln!(
            "✅ All {} checks healthy ({:.1}s)",
            result.checks.len(),
            result.duration.as_secs_f64()
        );
    } else if result.any_down() {
        let down = result
            .checks
            .iter()
            .filter(|c| matches!(c.status, CanaryStatus::Down(_)))
            .count();
        eprintln!(
            "❌ {down}/{} checks DOWN ({:.1}s)",
            result.checks.len(),
            result.duration.as_secs_f64()
        );
    } else {
        eprintln!(
            "⚠️  Some checks degraded ({:.1}s)",
            result.duration.as_secs_f64()
        );
    }
}
