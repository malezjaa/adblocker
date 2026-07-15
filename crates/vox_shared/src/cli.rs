use clap::builder::styling::{AnsiColor, Effects, Styles};

pub fn styles() -> Styles {
  Styles::styled()
    .header(AnsiColor::BrightCyan.on_default().effects(Effects::BOLD))
    .usage(AnsiColor::BrightCyan.on_default().effects(Effects::BOLD))
    .literal(AnsiColor::BrightGreen.on_default().effects(Effects::BOLD))
    .placeholder(AnsiColor::BrightYellow.on_default())
    .valid(AnsiColor::BrightGreen.on_default().effects(Effects::BOLD))
    .invalid(AnsiColor::BrightRed.on_default().effects(Effects::BOLD))
    .error(AnsiColor::BrightRed.on_default().effects(Effects::BOLD))
    .context(AnsiColor::BrightBlack.on_default())
    .context_value(AnsiColor::BrightYellow.on_default())
}
