use crate::canary::{CanaryStatus, HealthCheck};
use std::time::Instant;

pub async fn check(url: &str, timeout_secs: u64) -> HealthCheck {
    let start = Instant::now();

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .danger_accept_invalid_certs(false)
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return HealthCheck {
                name: format!("HTTP {url}"),
                status: CanaryStatus::Down(format!("client error: {e}")),
                latency_ms: None,
                details: None,
            };
        }
    };

    match client.get(url).send().await {
        Ok(resp) => {
            let latency = start.elapsed().as_millis() as u64;
            let status_code = resp.status();
            let content_length = resp.content_length();

            let details = format!(
                "status: {status_code}, size: {}",
                content_length
                    .map(|l| format!("{l} bytes"))
                    .unwrap_or_else(|| "unknown".into())
            );

            if status_code.is_success() {
                let status = if latency > 3000 {
                    CanaryStatus::Degraded(format!("slow response: {latency}ms"))
                } else {
                    CanaryStatus::Healthy
                };

                HealthCheck {
                    name: format!("HTTP {url}"),
                    status,
                    latency_ms: Some(latency),
                    details: Some(details),
                }
            } else if status_code.is_server_error() {
                HealthCheck {
                    name: format!("HTTP {url}"),
                    status: CanaryStatus::Down(format!("server error: {status_code}")),
                    latency_ms: Some(latency),
                    details: Some(details),
                }
            } else {
                HealthCheck {
                    name: format!("HTTP {url}"),
                    status: CanaryStatus::Degraded(format!("unexpected status: {status_code}")),
                    latency_ms: Some(latency),
                    details: Some(details),
                }
            }
        }
        Err(e) => {
            let latency = start.elapsed().as_millis() as u64;

            let msg = if e.is_timeout() {
                format!("timeout after {timeout_secs}s")
            } else if e.is_connect() {
                "connection refused".into()
            } else {
                format!("{e}")
            };

            HealthCheck {
                name: format!("HTTP {url}"),
                status: CanaryStatus::Down(msg),
                latency_ms: Some(latency),
                details: None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check_invalid_url() {
        let result = check("http://127.0.0.1:1", 2).await;
        assert!(matches!(result.status, CanaryStatus::Down(_)));
    }

    #[tokio::test]
    async fn test_check_nonexistent_host() {
        let result = check("http://this-host-does-not-exist.invalid", 3).await;
        assert!(matches!(result.status, CanaryStatus::Down(_)));
    }
}
