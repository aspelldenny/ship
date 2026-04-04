pub mod display;
pub mod store;

use crate::config::LearnConfig;
use crate::error::Result;

pub fn add(config: &LearnConfig, project: &str, message: &str, tags: &[String]) -> Result<()> {
    let learning = store::Learning::new(project, message, tags);
    let path = resolve_path(config, project);
    store::append(&path, &learning)?;
    eprintln!("✅ Saved: {}", learning.message);
    if !learning.tags.is_empty() {
        eprintln!("   tags: {}", learning.tags.join(", "));
    }
    Ok(())
}

pub fn search(config: &LearnConfig, project: &str, query: &str) -> Result<()> {
    let path = resolve_path(config, project);
    let results = store::search(&path, query)?;

    if results.is_empty() {
        eprintln!("No learnings found for \"{query}\"");
        return Ok(());
    }

    eprintln!("Found {} learnings:\n", results.len());
    for l in &results {
        display::print_learning(l);
    }
    Ok(())
}

pub fn list(config: &LearnConfig, project: &str, recent: usize) -> Result<()> {
    let path = resolve_path(config, project);
    let all = store::load_all(&path)?;

    if all.is_empty() {
        eprintln!("No learnings recorded yet.");
        return Ok(());
    }

    let items: Vec<_> = all.iter().rev().take(recent).collect();
    eprintln!("{} learnings (showing last {}):\n", all.len(), items.len());
    for l in &items {
        display::print_learning(l);
    }
    Ok(())
}

pub fn prune(config: &LearnConfig, project: &str) -> Result<()> {
    let path = resolve_path(config, project);
    let all = store::load_all(&path)?;

    if all.is_empty() {
        eprintln!("Nothing to prune.");
        return Ok(());
    }

    // Remove duplicates (same message)
    let before = all.len();
    let mut seen = std::collections::HashSet::new();
    let deduped: Vec<_> = all
        .into_iter()
        .filter(|l| seen.insert(l.message.clone()))
        .collect();
    let after = deduped.len();

    store::write_all(&path, &deduped)?;
    eprintln!("Pruned {} duplicates ({before} → {after})", before - after);
    Ok(())
}

pub fn resolve_path_pub(config: &LearnConfig, project: &str) -> std::path::PathBuf {
    resolve_path(config, project)
}

fn resolve_path(config: &LearnConfig, project: &str) -> std::path::PathBuf {
    // Per-project learnings if .ship/ exists locally
    let local = std::path::PathBuf::from(&config.project_dir);
    if local.parent().map(|p| p.exists()).unwrap_or(false) {
        return local.join(format!("{project}.jsonl"));
    }

    // Global learnings directory
    let global = shellexpand(config.dir.clone());
    let dir = std::path::PathBuf::from(&global);
    std::fs::create_dir_all(&dir).ok();
    dir.join(format!("{project}.jsonl"))
}

fn shellexpand(path: String) -> String {
    if let Some(rest) = path.strip_prefix("~/")
        && let Some(home) = dirs_home()
    {
        return format!("{}/{rest}", home);
    }
    path
}

fn dirs_home() -> Option<String> {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .ok()
}
