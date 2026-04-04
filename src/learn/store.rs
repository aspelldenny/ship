use crate::error::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Learning {
    pub id: String,
    pub timestamp: String,
    pub project: String,
    pub message: String,
    pub tags: Vec<String>,
}

impl Learning {
    pub fn new(project: &str, message: &str, tags: &[String]) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now().to_rfc3339(),
            project: project.to_string(),
            message: message.to_string(),
            tags: tags.to_vec(),
        }
    }

    pub fn matches(&self, query: &str) -> bool {
        let q = query.to_lowercase();
        self.message.to_lowercase().contains(&q)
            || self.tags.iter().any(|t| t.to_lowercase().contains(&q))
            || self.project.to_lowercase().contains(&q)
    }
}

pub fn append(path: &Path, learning: &Learning) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let line = serde_json::to_string(learning)?;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;

    use std::io::Write;
    writeln!(file, "{line}")?;
    Ok(())
}

pub fn load_all(path: &Path) -> Result<Vec<Learning>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(path)?;
    let learnings: Vec<Learning> = content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect();

    Ok(learnings)
}

pub fn search(path: &Path, query: &str) -> Result<Vec<Learning>> {
    let all = load_all(path)?;
    let results: Vec<Learning> = all.into_iter().filter(|l| l.matches(query)).collect();
    Ok(results)
}

pub fn write_all(path: &Path, learnings: &[Learning]) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let content: String = learnings
        .iter()
        .map(|l| serde_json::to_string(l).unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n");

    std::fs::write(path, format!("{content}\n"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_learning_new() {
        let l = Learning::new("tarot", "always run migrations", &["deploy".into()]);
        assert_eq!(l.project, "tarot");
        assert_eq!(l.message, "always run migrations");
        assert_eq!(l.tags, vec!["deploy"]);
        assert!(!l.id.is_empty());
    }

    #[test]
    fn test_matches_message() {
        let l = Learning::new("tarot", "Prisma needs ALTER TABLE after deploy", &[]);
        assert!(l.matches("prisma"));
        assert!(l.matches("ALTER"));
        assert!(!l.matches("flask"));
    }

    #[test]
    fn test_matches_tag() {
        let l = Learning::new("tarot", "something", &["docker".into(), "deploy".into()]);
        assert!(l.matches("docker"));
        assert!(l.matches("deploy"));
    }

    #[test]
    fn test_append_and_load() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.jsonl");

        let l1 = Learning::new("tarot", "first learning", &[]);
        let l2 = Learning::new("jarvis", "second learning", &["bot".into()]);

        append(&path, &l1).unwrap();
        append(&path, &l2).unwrap();

        let all = load_all(&path).unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].message, "first learning");
        assert_eq!(all[1].message, "second learning");
    }

    #[test]
    fn test_search() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.jsonl");

        append(&path, &Learning::new("tarot", "docker compose down", &[])).unwrap();
        append(&path, &Learning::new("tarot", "prisma migration", &[])).unwrap();
        append(&path, &Learning::new("jarvis", "docker restart", &[])).unwrap();

        let results = search(&path, "docker").unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_write_all_prune() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.jsonl");

        let l1 = Learning::new("tarot", "duplicate", &[]);
        let l2 = Learning::new("tarot", "unique", &[]);

        append(&path, &l1).unwrap();
        append(&path, &Learning::new("tarot", "duplicate", &[])).unwrap();
        append(&path, &l2).unwrap();

        let all = load_all(&path).unwrap();
        assert_eq!(all.len(), 3);

        // Deduplicate
        let mut seen = std::collections::HashSet::new();
        let deduped: Vec<_> = all
            .into_iter()
            .filter(|l| seen.insert(l.message.clone()))
            .collect();
        assert_eq!(deduped.len(), 2);

        write_all(&path, &deduped).unwrap();
        let reloaded = load_all(&path).unwrap();
        assert_eq!(reloaded.len(), 2);
    }

    #[test]
    fn test_load_nonexistent() {
        let result = load_all(Path::new("/nonexistent/file.jsonl")).unwrap();
        assert!(result.is_empty());
    }
}
