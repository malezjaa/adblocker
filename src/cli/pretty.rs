use yansi::Paint;

pub fn print_separator(width: usize) {
  println!("  {}", "─".repeat(width).dim());
}

pub fn print_success(label: &str) {
  println!();
  println!("  {} {}", "✓".green().bold(), label.green().bold());
}

pub fn print_warning(msg: &str) {
  println!("  {} {}", "⚠".bright_yellow(), msg.rgb(251, 191, 36));
}

pub fn print_error(msg: &str) {
  println!("  {} {}", "✗".red().bold(), msg.bright_red().bold());
}

pub fn print_field(key: &str, value: impl std::fmt::Display) {
  println!("  {}  {}", key.dim(), value);
}
