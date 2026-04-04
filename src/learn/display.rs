use super::store::Learning;

pub fn print_learning(l: &Learning) {
    let date = l.timestamp.split('T').next().unwrap_or(&l.timestamp);

    let tags = if l.tags.is_empty() {
        String::new()
    } else {
        format!(" [{}]", l.tags.join(", "))
    };

    eprintln!("  • {}{}", l.message, tags);
    eprintln!("    {} | {}", l.project, date);
    eprintln!();
}
