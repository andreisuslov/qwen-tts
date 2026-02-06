use colored::Colorize;

pub fn status(label: &str, message: &str) {
    eprintln!("{} {}", label.cyan().bold(), message);
}

pub fn success(message: &str) {
    eprintln!("{} {}", "Done!".green().bold(), message);
}

pub fn warn(message: &str) {
    eprintln!("{} {}", "Warning:".yellow().bold(), message);
}

pub fn error(message: &str) {
    eprintln!("{} {}", "Error:".red().bold(), message);
}
