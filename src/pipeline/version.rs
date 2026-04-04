use crate::config::Config;
use crate::error::Result;
use crate::pipeline::{StepResult, StepStatus};
use std::process::Command;
use std::time::Instant;

pub fn run(config: &Config, override_bump: Option<&str>) -> Result<StepResult> {
    let start = Instant::now();

    let version_file = resolve_version_file(config);
    let Some(path) = version_file else {
        return Ok(StepResult {
            name: "Version".into(),
            status: StepStatus::Skip("no VERSION file found".into()),
            duration: start.elapsed(),
            output: None,
        });
    };

    let current = std::fs::read_to_string(&path)
        .unwrap_or_default()
        .trim()
        .to_string();

    let bump = if let Some(b) = override_bump {
        b.to_string()
    } else {
        auto_detect_bump(config)?
    };

    let new_version = bump_version(&current, &bump);
    std::fs::write(&path, format!("{new_version}\n"))?;

    Ok(StepResult {
        name: "Version".into(),
        status: StepStatus::Pass,
        duration: start.elapsed(),
        output: Some(format!("{current} → {new_version} ({bump})")),
    })
}

fn resolve_version_file(config: &Config) -> Option<String> {
    if let Some(f) = &config.version.file {
        return Some(f.clone());
    }

    for candidate in ["VERSION", "version.txt"] {
        if std::path::Path::new(candidate).exists() {
            return Some(candidate.into());
        }
    }

    None
}

fn auto_detect_bump(config: &Config) -> Result<String> {
    let output = Command::new("git")
        .args([
            "diff",
            "--stat",
            &format!("{}...HEAD", config.base_branch),
        ])
        .output()?;

    let stat = String::from_utf8_lossy(&output.stdout);

    // Count insertions + deletions
    let total_lines: u32 = stat
        .lines()
        .last()
        .and_then(|line| {
            let nums: Vec<u32> = line
                .split_whitespace()
                .filter_map(|w| w.parse().ok())
                .collect();
            nums.get(1..).map(|s| s.iter().sum())
        })
        .unwrap_or(0);

    let thresholds = &config.version.auto_thresholds;
    let bump = if total_lines < thresholds.patch {
        "patch"
    } else if total_lines < thresholds.minor {
        "minor"
    } else {
        "minor" // > 500 lines, could be major but default to minor
    };

    Ok(bump.into())
}

fn bump_version(current: &str, bump: &str) -> String {
    let parts: Vec<u32> = current
        .split('.')
        .filter_map(|p| p.parse().ok())
        .collect();

    let (major, minor, patch) = match parts.as_slice() {
        [ma, mi, pa, ..] => (*ma, *mi, *pa),
        [ma, mi] => (*ma, *mi, 0),
        [ma] => (*ma, 0, 0),
        _ => (0, 1, 0),
    };

    match bump {
        "major" => format!("{}.0.0", major + 1),
        "minor" => format!("{}.{}.0", major, minor + 1),
        "patch" => format!("{}.{}.{}", major, minor, patch + 1),
        _ => format!("{}.{}.{}", major, minor, patch + 1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bump_patch() {
        assert_eq!(bump_version("1.2.3", "patch"), "1.2.4");
    }

    #[test]
    fn test_bump_minor() {
        assert_eq!(bump_version("1.2.3", "minor"), "1.3.0");
    }

    #[test]
    fn test_bump_major() {
        assert_eq!(bump_version("1.2.3", "major"), "2.0.0");
    }

    #[test]
    fn test_bump_from_two_parts() {
        assert_eq!(bump_version("1.2", "patch"), "1.2.1");
    }

    #[test]
    fn test_bump_from_empty() {
        assert_eq!(bump_version("", "patch"), "0.1.1");
    }
}
