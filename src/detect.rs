use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub enum ProjectStack {
    Rust,
    NextJs,
    Flask,
    Python,
    Node,
    Unknown,
}

impl ProjectStack {
    /// Auto-detect project stack from filesystem
    pub fn detect(root: &Path) -> Self {
        if root.join("Cargo.toml").exists() {
            return Self::Rust;
        }

        if root.join("package.json").exists() {
            if root.join("next.config.mjs").exists()
                || root.join("next.config.js").exists()
                || root.join("next.config.ts").exists()
            {
                return Self::NextJs;
            }
            return Self::Node;
        }

        if root.join("requirements.txt").exists() {
            if let Ok(content) = std::fs::read_to_string(root.join("requirements.txt")) {
                if content.to_lowercase().contains("flask") {
                    return Self::Flask;
                }
            }
            return Self::Python;
        }

        if root.join("pyproject.toml").exists() {
            return Self::Python;
        }

        Self::Unknown
    }

    /// Default test command for this stack
    pub fn test_command(&self) -> Option<&str> {
        match self {
            Self::Rust => Some("cargo test"),
            Self::NextJs => Some("pnpm test --run"),
            Self::Flask => Some("python -m pytest tests/ -x"),
            Self::Python => Some("python -m pytest"),
            Self::Node => Some("npm test"),
            Self::Unknown => None,
        }
    }

    /// Display name
    pub fn name(&self) -> &str {
        match self {
            Self::Rust => "Rust",
            Self::NextJs => "Next.js",
            Self::Flask => "Flask",
            Self::Python => "Python",
            Self::Node => "Node.js",
            Self::Unknown => "Unknown",
        }
    }
}

impl std::fmt::Display for ProjectStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_detect_rust() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
        assert_eq!(ProjectStack::detect(dir.path()), ProjectStack::Rust);
    }

    #[test]
    fn test_detect_nextjs() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("package.json"), "{}").unwrap();
        std::fs::write(dir.path().join("next.config.mjs"), "export default {}").unwrap();
        assert_eq!(ProjectStack::detect(dir.path()), ProjectStack::NextJs);
    }

    #[test]
    fn test_detect_flask() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("requirements.txt"), "flask==3.0\nsqlalchemy").unwrap();
        assert_eq!(ProjectStack::detect(dir.path()), ProjectStack::Flask);
    }

    #[test]
    fn test_detect_python() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("pyproject.toml"), "[project]\nname = \"test\"").unwrap();
        assert_eq!(ProjectStack::detect(dir.path()), ProjectStack::Python);
    }

    #[test]
    fn test_detect_node() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("package.json"), "{}").unwrap();
        assert_eq!(ProjectStack::detect(dir.path()), ProjectStack::Node);
    }

    #[test]
    fn test_detect_unknown() {
        let dir = TempDir::new().unwrap();
        assert_eq!(ProjectStack::detect(dir.path()), ProjectStack::Unknown);
    }

    #[test]
    fn test_rust_test_command() {
        assert_eq!(ProjectStack::Rust.test_command(), Some("cargo test"));
    }

    #[test]
    fn test_unknown_test_command() {
        assert_eq!(ProjectStack::Unknown.test_command(), None);
    }
}
