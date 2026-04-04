use std::time::Duration;

pub fn step_pass(name: &str, detail: &str, duration: Duration) {
    eprintln!(
        "  ✅ {} — {} ({:.1}s)",
        name,
        detail,
        duration.as_secs_f64()
    );
}

pub fn step_fail(name: &str, detail: &str, duration: Duration) {
    eprintln!(
        "  ❌ {} — {} ({:.1}s)",
        name,
        detail,
        duration.as_secs_f64()
    );
}

pub fn step_warn(name: &str, detail: &str, duration: Duration) {
    eprintln!(
        "  ⚠️  {} — {} ({:.1}s)",
        name,
        detail,
        duration.as_secs_f64()
    );
}

pub fn step_skip(name: &str, reason: &str) {
    eprintln!("  ⏭️  {} — {}", name, reason);
}

pub fn header(msg: &str) {
    eprintln!("\n🚀 {}", msg);
    eprintln!("{}", "─".repeat(50));
}

pub fn summary(total: usize, _passed: usize, failed: usize, duration: Duration) {
    eprintln!("{}", "─".repeat(50));
    if failed == 0 {
        eprintln!(
            "✅ All {} steps passed ({:.1}s)",
            total,
            duration.as_secs_f64()
        );
    } else {
        eprintln!(
            "❌ {}/{} steps failed ({:.1}s)",
            failed,
            total,
            duration.as_secs_f64()
        );
    }
}

pub fn pr_url(url: &str) {
    eprintln!("\n🔗 PR: {}", url);
}
