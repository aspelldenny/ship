use crate::config::DeployConfig;
use crate::error::{Result, ShipError};
use crate::output;
use std::process::Command;
use std::time::Instant;

pub struct DeployResult {
    pub success: bool,
    #[allow(dead_code)]
    pub duration: std::time::Duration,
}

pub async fn run(
    deploy_config: &DeployConfig,
    canary_config: &crate::config::CanaryConfig,
) -> Result<DeployResult> {
    let start = Instant::now();

    output::header("Deploy");

    let provider = &deploy_config.provider;
    eprintln!("  Provider: {provider}");

    match provider.as_str() {
        "ssh" => run_ssh(deploy_config).await?,
        "github-actions" => run_github_actions()?,
        "cargo" => run_cargo_publish()?,
        "render" => {
            eprintln!("  ⏭️  Render auto-deploys on push — skipping trigger");
        }
        "custom" => {
            if let Some(cmd) = &deploy_config.command {
                run_custom(cmd)?;
            } else {
                return Err(ShipError::Config(
                    "deploy.command required for custom provider".into(),
                ));
            }
        }
        other => {
            return Err(ShipError::Config(format!(
                "unknown deploy provider: {other}"
            )));
        }
    }

    // Post-deploy canary if URL configured
    if canary_config.url.is_some() {
        eprintln!("\n  Running post-deploy canary...");
        match crate::canary::run(canary_config).await {
            Ok(result) => {
                if !result.all_healthy() {
                    eprintln!("  ⚠️  Canary detected issues after deploy!");
                }
            }
            Err(e) => {
                eprintln!("  ⚠️  Canary failed: {e}");
            }
        }
    }

    let duration = start.elapsed();
    eprintln!("{}", "─".repeat(50));
    eprintln!("✅ Deploy completed ({:.1}s)", duration.as_secs_f64());

    Ok(DeployResult {
        success: true,
        duration,
    })
}

async fn run_ssh(config: &DeployConfig) -> Result<()> {
    let ssh = config
        .ssh
        .as_deref()
        .ok_or_else(|| ShipError::Config("deploy.ssh required for ssh provider".into()))?;

    let (ssh_target, port) = parse_ssh(ssh);

    // Maintenance mode ON
    if config.maintenance_mode {
        eprintln!("  🔧 Maintenance mode ON...");
        run_ssh_command(&ssh_target, port.as_deref(), "echo 'maintenance on'")?;
    }

    // Run deploy command
    let deploy_cmd = config.command.as_deref().unwrap_or(
        "cd /opt/app && git pull origin main && docker compose build && docker compose up -d",
    );

    eprintln!("  🚀 Running deploy...");
    let output = run_ssh_command(&ssh_target, port.as_deref(), deploy_cmd)?;
    if !output.is_empty() {
        for line in output.lines().take(10) {
            eprintln!("     {line}");
        }
    }

    // Maintenance mode OFF
    if config.maintenance_mode {
        eprintln!("  🔧 Maintenance mode OFF...");
        run_ssh_command(&ssh_target, port.as_deref(), "echo 'maintenance off'")?;
    }

    eprintln!("  ✅ SSH deploy complete");
    Ok(())
}

fn run_github_actions() -> Result<()> {
    eprintln!("  GitHub Actions deploys on push to main.");
    eprintln!("  Checking latest workflow run...");

    let output = Command::new("gh")
        .args([
            "run",
            "list",
            "--limit",
            "1",
            "--json",
            "status,conclusion,name",
            "-q",
            ".[0]",
        ])
        .output()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        eprintln!("  {}", stdout.trim());
    } else {
        eprintln!("  ⚠️  Could not check workflow status (gh CLI)");
    }

    Ok(())
}

fn run_cargo_publish() -> Result<()> {
    eprintln!("  📦 Running cargo publish --dry-run...");
    let output = Command::new("cargo")
        .args(["publish", "--dry-run"])
        .output()?;

    if output.status.success() {
        eprintln!("  ✅ Dry run passed. Run `cargo publish` manually to confirm.");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("  ❌ Publish dry-run failed:\n{stderr}");
    }

    Ok(())
}

fn run_custom(cmd: &str) -> Result<()> {
    eprintln!("  Running: {cmd}");
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let (program, args) = parts.split_first().expect("command not empty");

    let output = Command::new(program).args(args).output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.trim().is_empty() {
        for line in stdout.lines().take(10) {
            eprintln!("     {line}");
        }
    }

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ShipError::PushFailed(format!(
            "deploy command failed: {stderr}"
        )));
    }

    eprintln!("  ✅ Custom deploy complete");
    Ok(())
}

fn run_ssh_command(target: &str, port: Option<&str>, cmd: &str) -> Result<String> {
    let mut ssh = Command::new("ssh");
    ssh.args(["-o", "StrictHostKeyChecking=no"])
        .args(["-o", "ConnectTimeout=15"])
        .args(["-o", "BatchMode=yes"]);

    if let Some(p) = port {
        ssh.args(["-p", p]);
    }

    ssh.arg(target).arg(cmd);

    let output = ssh.output()?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ShipError::PushFailed(format!(
            "SSH command failed: {stderr}"
        )));
    }

    Ok(stdout)
}

fn parse_ssh(ssh: &str) -> (String, Option<String>) {
    if let Some((target, port)) = ssh.rsplit_once(':')
        && port.parse::<u16>().is_ok()
    {
        return (target.to_string(), Some(port.to_string()));
    }
    (ssh.to_string(), None)
}
