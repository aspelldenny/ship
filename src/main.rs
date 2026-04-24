mod canary;
mod config;
mod deploy;
mod detect;
mod error;
mod learn;
mod mcp;
mod note;
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

    /// Post-deploy health check
    Canary {
        /// Health check URL (overrides config)
        #[arg(long)]
        url: Option<String>,
        /// Docker container name (overrides config)
        #[arg(long)]
        docker: Option<String>,
        /// SSH target for Docker check (overrides config)
        #[arg(long)]
        ssh: Option<String>,
        /// Health check timeout in seconds
        #[arg(long, default_value = "30")]
        timeout: u64,
    },
    /// Deploy to production
    Deploy {
        /// Override deploy provider
        #[arg(long)]
        provider: Option<String>,
        /// Override SSH target
        #[arg(long)]
        ssh: Option<String>,
        /// Override deploy command
        #[arg(long)]
        command: Option<String>,
        /// Skip post-deploy canary check
        #[arg(long)]
        skip_canary: bool,
    },

    /// Manage cross-project learnings
    Learn {
        #[command(subcommand)]
        action: LearnAction,
    },
    /// Export a ship note to the Obsidian vault (per-phiếu log)
    Note {
        /// Project slug (overrides config, else cwd dirname)
        #[arg(long)]
        project: Option<String>,
        /// Ticket ID for the note frontmatter
        #[arg(long)]
        ticket: Option<String>,
        /// Free-form learnings line; omitted section if absent
        #[arg(long)]
        message: Option<String>,
        /// Vault path (overrides env OBSIDIAN_VAULT_PATH and config)
        #[arg(long)]
        vault_path: Option<String>,
    },
    /// Start MCP server for Claude integration
    Serve,
}

#[derive(Subcommand)]
enum LearnAction {
    /// Add a new learning
    Add {
        /// The learning message
        message: String,
        /// Tags for categorization
        #[arg(long, short, value_delimiter = ',')]
        tags: Vec<String>,
    },
    /// Search learnings by keyword
    Search {
        /// Search query
        query: String,
    },
    /// List recent learnings
    List {
        /// Number of recent items to show
        #[arg(long, short, default_value = "10")]
        recent: usize,
    },
    /// Remove duplicate learnings
    Prune,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
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
                    let failed = result.has_failures();
                    if !failed && config.obsidian.auto_log {
                        match note::run(&config.obsidian, note::NoteOptions::default()) {
                            note::NoteOutcome::Written(p) => {
                                eprintln!("📝 Logged to vault: {}", p.display());
                            }
                            note::NoteOutcome::Skipped(reason) => {
                                eprintln!("⚠️  Vault log skipped: {reason}");
                            }
                        }
                    }
                    if failed { 1 } else { 0 }
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

        Some(Commands::Canary {
            url,
            docker,
            ssh,
            timeout,
        }) => {
            let mut canary_config = config.canary.clone();

            // CLI args override config
            if let Some(u) = url {
                canary_config.url = Some(u);
            }
            if let Some(d) = docker {
                canary_config.docker_container = Some(d);
            }
            if let Some(s) = ssh {
                canary_config.ssh = Some(s);
            }
            canary_config.timeout_secs = timeout;

            match canary::run(&canary_config).await {
                Ok(result) => {
                    if result.all_healthy() {
                        0
                    } else {
                        1
                    }
                }
                Err(e) => {
                    eprintln!("❌ Canary error: {e}");
                    1
                }
            }
        }

        Some(Commands::Deploy {
            provider,
            ssh,
            command,
            skip_canary,
        }) => {
            let mut deploy_config = config.deploy.clone();
            if let Some(p) = provider {
                deploy_config.provider = p;
            }
            if let Some(s) = ssh {
                deploy_config.ssh = Some(s);
            }
            if let Some(c) = command {
                deploy_config.command = Some(c);
            }

            let canary_config = if skip_canary {
                crate::config::CanaryConfig {
                    checks: vec![],
                    ..config.canary.clone()
                }
            } else {
                config.canary.clone()
            };

            match deploy::run(&deploy_config, &canary_config).await {
                Ok(result) => {
                    if result.success {
                        0
                    } else {
                        1
                    }
                }
                Err(e) => {
                    eprintln!("❌ Deploy error: {e}");
                    1
                }
            }
        }

        Some(Commands::Note {
            project,
            ticket,
            message,
            vault_path,
        }) => {
            let opts = note::NoteOptions {
                project,
                ticket,
                message,
                vault_path,
            };
            match note::run(&config.obsidian, opts) {
                note::NoteOutcome::Written(p) => {
                    println!("{}", p.display());
                    0
                }
                note::NoteOutcome::Skipped(reason) => {
                    eprintln!("⚠️  Vault log skipped: {reason}");
                    0
                }
            }
        }

        Some(Commands::Serve) => match mcp::server::serve(config).await {
            Ok(_) => 0,
            Err(e) => {
                eprintln!("❌ MCP server error: {e}");
                1
            }
        },

        Some(Commands::Learn { action }) => {
            let project = config.project_name();
            match action {
                LearnAction::Add { message, tags } => {
                    match learn::add(&config.learn, &project, &message, &tags) {
                        Ok(_) => 0,
                        Err(e) => {
                            eprintln!("❌ Learn error: {e}");
                            1
                        }
                    }
                }
                LearnAction::Search { query } => {
                    match learn::search(&config.learn, &project, &query) {
                        Ok(_) => 0,
                        Err(e) => {
                            eprintln!("❌ Learn error: {e}");
                            1
                        }
                    }
                }
                LearnAction::List { recent } => {
                    match learn::list(&config.learn, &project, recent) {
                        Ok(_) => 0,
                        Err(e) => {
                            eprintln!("❌ Learn error: {e}");
                            1
                        }
                    }
                }
                LearnAction::Prune => match learn::prune(&config.learn, &project) {
                    Ok(_) => 0,
                    Err(e) => {
                        eprintln!("❌ Learn error: {e}");
                        1
                    }
                },
            }
        }
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
