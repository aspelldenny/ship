use crate::error::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    pub name: Option<String>,
    pub stack: Option<String>,
    pub base_branch: String,

    pub test: TestConfig,
    pub docs_gate: DocsGateConfig,
    pub version: VersionConfig,
    pub changelog: ChangelogConfig,
    pub pr: PrConfig,
    pub canary: CanaryConfig,
    pub learn: LearnConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct TestConfig {
    pub command: Option<String>,
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct DocsGateConfig {
    pub enabled: bool,
    pub blocking: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct VersionConfig {
    pub file: Option<String>,
    pub strategy: String,
    pub auto_thresholds: AutoThresholds,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AutoThresholds {
    pub patch: u32,
    pub minor: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ChangelogConfig {
    pub file: String,
    pub style: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct PrConfig {
    pub template: String,
    pub draft: bool,
    pub labels: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct CanaryConfig {
    pub url: Option<String>,
    pub docker_container: Option<String>,
    pub ssh: Option<String>,
    pub timeout_secs: u64,
    pub checks: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct LearnConfig {
    pub dir: String,
    pub project_dir: String,
}

// --- Defaults ---

impl Default for Config {
    fn default() -> Self {
        Self {
            name: None,
            stack: None,
            base_branch: "main".into(),
            test: TestConfig::default(),
            docs_gate: DocsGateConfig::default(),
            version: VersionConfig::default(),
            changelog: ChangelogConfig::default(),
            pr: PrConfig::default(),
            canary: CanaryConfig::default(),
            learn: LearnConfig::default(),
        }
    }
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            command: None,
            timeout_secs: 300,
        }
    }
}

impl Default for DocsGateConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            blocking: false,
        }
    }
}

impl Default for VersionConfig {
    fn default() -> Self {
        Self {
            file: None,
            strategy: "auto".into(),
            auto_thresholds: AutoThresholds::default(),
        }
    }
}

impl Default for AutoThresholds {
    fn default() -> Self {
        Self {
            patch: 50,
            minor: 500,
        }
    }
}

impl Default for ChangelogConfig {
    fn default() -> Self {
        Self {
            file: "docs/CHANGELOG.md".into(),
            style: "grouped".into(),
        }
    }
}

impl Default for PrConfig {
    fn default() -> Self {
        Self {
            template: "default".into(),
            draft: false,
            labels: vec![],
        }
    }
}

impl Default for CanaryConfig {
    fn default() -> Self {
        Self {
            url: None,
            docker_container: None,
            ssh: None,
            timeout_secs: 30,
            checks: vec!["http".into()],
        }
    }
}

impl Default for LearnConfig {
    fn default() -> Self {
        Self {
            dir: "~/.ship/learnings".into(),
            project_dir: ".ship/learnings".into(),
        }
    }
}

// --- Loading ---

impl Config {
    pub fn load(config_path: Option<&Path>) -> Result<Self> {
        let path = config_path
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(".ship.toml"));

        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    pub fn project_name(&self) -> String {
        self.name.clone().unwrap_or_else(|| {
            std::env::current_dir()
                .ok()
                .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
                .unwrap_or_else(|| "unknown".into())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.base_branch, "main");
        assert_eq!(config.test.timeout_secs, 300);
        assert!(config.docs_gate.enabled);
        assert!(!config.docs_gate.blocking);
        assert_eq!(config.version.auto_thresholds.patch, 50);
        assert_eq!(config.changelog.file, "docs/CHANGELOG.md");
    }

    #[test]
    fn test_load_missing_config_uses_defaults() {
        let config = Config::load(Some(Path::new("/nonexistent/.ship.toml"))).unwrap();
        assert_eq!(config.base_branch, "main");
    }

    #[test]
    fn test_load_valid_toml() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join(".ship.toml");
        std::fs::write(
            &path,
            r#"
name = "tarot"
base_branch = "develop"

[test]
command = "pnpm test --run"
timeout_secs = 120
"#,
        )
        .unwrap();

        let config = Config::load(Some(&path)).unwrap();
        assert_eq!(config.name, Some("tarot".into()));
        assert_eq!(config.base_branch, "develop");
        assert_eq!(config.test.command, Some("pnpm test --run".into()));
        assert_eq!(config.test.timeout_secs, 120);
    }

    #[test]
    fn test_project_name_from_config() {
        let config = Config {
            name: Some("jarvis".into()),
            ..Config::default()
        };
        assert_eq!(config.project_name(), "jarvis");
    }
}
