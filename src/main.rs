mod config;
mod detect;
mod error;
mod output;
mod pipeline;

use clap::{Parser, Subcommand};
use config::Config;
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(
    name = "ship",
    about = "Automated release workflow — test, commit, push, PR in one command",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Custom config file path
    #[arg(long, global = true)]
    config: Option<PathBuf>,

    /// Show verbose output
    #[arg(long, short, global = true)]
    verbose: bool,

    /// Simulate without side effects
    #[arg(long)]
    dry_run: bool,

    /// Skip test step
    #[arg(long)]
    skip_tests: bool,

    /// Skip docs-gate step
    #[arg(long)]
    skip_docs_gate: bool,

    /// Override version bump level
    #[arg(long, value_parser = ["patch", "minor", "major"])]
    bump: Option<String>,

    /// Skip PR creation (commit + push only)
    #[arg(long)]
    no_pr: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Run pre-flight checks only (test + docs-gate, no commit)
    Check {
        #[arg(long)]
        skip_tests: bool,
        #[arg(long)]
        skip_docs_gate: bool,
    },

    /// Auto-detect project and generate .ship.toml
    Init,
    // Phase 2
    // /// Post-deploy health check
    // Canary { ... },

    // Phase 3
    // /// Manage cross-project learnings
    // Learn { ... },
    // /// Start MCP server
    // Serve,
}

fn main() {
    let cli = Cli::parse();

    let config = match Config::load(cli.config.as_deref()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ {e}");
            process::exit(2);
        }
    };

    let exit_code = match cli.command {
        None => {
            // Default: full ship pipeline
            let opts = pipeline::PipelineOptions {
                dry_run: cli.dry_run,
                skip_tests: cli.skip_tests,
                skip_docs_gate: cli.skip_docs_gate,
                bump: cli.bump,
                no_pr: cli.no_pr,
                verbose: cli.verbose,
            };

            match pipeline::run(&config, &opts) {
                Ok(result) => {
                    if result.has_failures() {
                        1
                    } else {
                        0
                    }
                }
                Err(e) => {
                    eprintln!("❌ Pipeline error: {e}");
                    1
                }
            }
        }

        Some(Commands::Check {
            skip_tests,
            skip_docs_gate,
        }) => {
            let opts = pipeline::PipelineOptions {
                dry_run: true,
                skip_tests,
                skip_docs_gate,
                bump: None,
                no_pr: true,
                verbose: cli.verbose,
            };

            match pipeline::check(&config, &opts) {
                Ok(result) => {
                    if result.has_failures() {
                        1
                    } else {
                        0
                    }
                }
                Err(e) => {
                    eprintln!("❌ Check error: {e}");
                    1
                }
            }
        }

        Some(Commands::Init) => match init_project() {
            Ok(_) => 0,
            Err(e) => {
                eprintln!("❌ Init error: {e}");
                1
            }
        },
    };

    process::exit(exit_code);
}

fn init_project() -> error::Result<()> {
    let cwd = std::env::current_dir()?;
    let stack = detect::ProjectStack::detect(&cwd);
    let project_name = cwd
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".into());

    eprintln!("🔍 Detected: {} ({})", project_name, stack);

    let test_cmd = stack.test_command().unwrap_or("echo 'no tests'");

    let toml_content = format!(
        r#"name = "{project_name}"
stack = "{stack_name}"
base_branch = "main"

[test]
command = "{test_cmd}"
timeout_secs = 300

[docs_gate]
enabled = true
blocking = false

[version]
# file = "VERSION"
strategy = "auto"

[changelog]
file = "docs/CHANGELOG.md"
style = "grouped"

[pr]
template = "default"
draft = false
# labels = ["ship"]
"#,
        stack_name = stack.name().to_lowercase(),
    );

    let config_path = cwd.join(".ship.toml");
    if config_path.exists() {
        eprintln!("⚠️  .ship.toml already exists, skipping");
        return Ok(());
    }

    std::fs::write(&config_path, &toml_content)?;
    eprintln!("✅ Created .ship.toml");
    eprintln!("\n{toml_content}");

    Ok(())
}
