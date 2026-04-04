// Phase 3: AI code review via OpenRouter/Claude API
// Stub for now — will integrate with AI when review config is enabled

use crate::pipeline::{StepResult, StepStatus};
use std::time::Instant;

pub fn _run() -> StepResult {
    let start = Instant::now();
    StepResult {
        name: "Review".into(),
        status: StepStatus::Skip("AI review not configured (Phase 3)".into()),
        duration: start.elapsed(),
        output: None,
    }
}
