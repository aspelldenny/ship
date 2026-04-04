pub mod changelog;
pub mod commit;
pub mod docs_gate;
pub mod pr;
pub mod preflight;
pub mod push;
pub mod test;
pub mod version;

use crate::config::Config;
use crate::error::Result;
use crate::output;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct StepResult {
    pub name: String,
    pub status: StepStatus,
    pub duration: Duration,
    pub output: Option<String>,
}

#[derive(Debug, Clone)]
pub enum StepStatus {
    Pass,
    Fail(String),
    Warn(String),
    Skip(String),
}

impl StepResult {
    pub fn is_failure(&self) -> bool {
        matches!(self.status, StepStatus::Fail(_))
    }
}

pub struct PipelineOptions {
    pub dry_run: bool,
    pub skip_tests: bool,
    pub skip_docs_gate: bool,
    pub bump: Option<String>,
    pub no_pr: bool,
    #[allow(dead_code)]
    pub verbose: bool,
}

pub struct PipelineResult {
    pub steps: Vec<StepResult>,
    pub duration: Duration,
    #[allow(dead_code)]
    pub pr_url: Option<String>,
}

impl PipelineResult {
    pub fn has_failures(&self) -> bool {
        self.steps.iter().any(|s| s.is_failure())
    }

    pub fn passed_count(&self) -> usize {
        self.steps
            .iter()
            .filter(|s| matches!(s.status, StepStatus::Pass))
            .count()
    }

    pub fn failed_count(&self) -> usize {
        self.steps.iter().filter(|s| s.is_failure()).count()
    }
}

/// Run full ship pipeline
pub fn run(config: &Config, opts: &PipelineOptions) -> Result<PipelineResult> {
    let start = Instant::now();
    let mut steps: Vec<StepResult> = Vec::new();

    output::header(&format!("Ship — {}", config.project_name()));

    // Step 1: Preflight
    let result = preflight::run(config)?;
    print_step(&result);
    let failed = result.is_failure();
    steps.push(result);
    if failed {
        return Ok(finish(steps, start, None));
    }

    // Step 2: Tests
    if opts.skip_tests {
        steps.push(skip("Test", "skipped (--skip-tests)"));
    } else {
        let result = test::run(config)?;
        print_step(&result);
        let failed = result.is_failure();
        steps.push(result);
        if failed {
            return Ok(finish(steps, start, None));
        }
    }

    // Step 3: docs-gate
    if opts.skip_docs_gate || !config.docs_gate.enabled {
        steps.push(skip("Docs Gate", "skipped"));
    } else {
        let result = docs_gate::run(config)?;
        print_step(&result);
        if result.is_failure() && config.docs_gate.blocking {
            steps.push(result);
            return Ok(finish(steps, start, None));
        }
        steps.push(result);
    }

    // Dry run stops here
    if opts.dry_run {
        steps.push(skip("Version", "dry run"));
        steps.push(skip("Changelog", "dry run"));
        steps.push(skip("Commit", "dry run"));
        steps.push(skip("Push", "dry run"));
        steps.push(skip("PR", "dry run"));
        return Ok(finish(steps, start, None));
    }

    // Step 5: Version bump
    let result = version::run(config, opts.bump.as_deref())?;
    print_step(&result);
    steps.push(result);

    // Step 6: Changelog
    let result = changelog::run(config)?;
    print_step(&result);
    steps.push(result);

    // Step 7: Commit
    let result = commit::run(config)?;
    print_step(&result);
    if result.is_failure() {
        steps.push(result);
        return Ok(finish(steps, start, None));
    }
    steps.push(result);

    // Step 8: Push
    let result = push::run()?;
    print_step(&result);
    if result.is_failure() {
        steps.push(result);
        return Ok(finish(steps, start, None));
    }
    steps.push(result);

    // Step 9: PR
    let mut pr_url = None;
    if opts.no_pr {
        steps.push(skip("PR", "skipped (--no-pr)"));
    } else {
        let result = pr::run(config, &steps)?;
        print_step(&result);
        if let StepStatus::Pass = &result.status {
            pr_url = result.output.clone();
        }
        steps.push(result);
    }

    if let Some(url) = &pr_url {
        output::pr_url(url);
    }

    Ok(finish(steps, start, pr_url))
}

/// Run check-only mode (no commit/push/PR)
pub fn check(config: &Config, opts: &PipelineOptions) -> Result<PipelineResult> {
    let start = Instant::now();
    let mut steps: Vec<StepResult> = Vec::new();

    output::header(&format!("Ship Check — {}", config.project_name()));

    let result = preflight::run(config)?;
    print_step(&result);
    steps.push(result);

    if !opts.skip_tests {
        let result = test::run(config)?;
        print_step(&result);
        steps.push(result);
    }

    if config.docs_gate.enabled && !opts.skip_docs_gate {
        let result = docs_gate::run(config)?;
        print_step(&result);
        steps.push(result);
    }

    Ok(finish(steps, start, None))
}

fn skip(name: &str, reason: &str) -> StepResult {
    output::step_skip(name, reason);
    StepResult {
        name: name.into(),
        status: StepStatus::Skip(reason.into()),
        duration: Duration::ZERO,
        output: None,
    }
}

fn print_step(result: &StepResult) {
    match &result.status {
        StepStatus::Pass => output::step_pass(
            &result.name,
            result.output.as_deref().unwrap_or("ok"),
            result.duration,
        ),
        StepStatus::Fail(msg) => output::step_fail(&result.name, msg, result.duration),
        StepStatus::Warn(msg) => output::step_warn(&result.name, msg, result.duration),
        StepStatus::Skip(msg) => output::step_skip(&result.name, msg),
    }
}

fn finish(steps: Vec<StepResult>, start: Instant, pr_url: Option<String>) -> PipelineResult {
    let result = PipelineResult {
        duration: start.elapsed(),
        pr_url,
        steps,
    };
    output::summary(
        result.steps.len(),
        result.passed_count(),
        result.failed_count(),
        result.duration,
    );
    result
}
