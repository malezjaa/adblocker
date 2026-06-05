use std::fmt;
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::{
  EnvFilter,
  fmt::{FmtContext, FormatEvent, FormatFields},
  registry::LookupSpan,
};
use yansi::Paint;

struct CustomFormatter;

impl<S, N> FormatEvent<S, N> for CustomFormatter
where
  S: Subscriber + for<'a> LookupSpan<'a>,
  N: for<'a> FormatFields<'a> + 'static,
{
  fn format_event(
    &self,
    ctx: &FmtContext<'_, S, N>,
    mut writer: tracing_subscriber::fmt::format::Writer<'_>,
    event: &Event<'_>,
  ) -> fmt::Result {
    let level = match *event.metadata().level() {
      Level::ERROR => Paint::red("[error]").bold(),
      Level::WARN => Paint::yellow("[warning]").bold(),
      Level::INFO => Paint::green("[info]").bold(),
      Level::DEBUG => Paint::blue("[debug]").bold(),
      Level::TRACE => Paint::magenta("[trace]").bold(),
    };
    write!(writer, "{} ", level)?;

    ctx.field_format().format_fields(writer.by_ref(), event)?;
    writeln!(writer)
  }
}

pub fn setup_logger(verbose: bool) {
  tracing_subscriber::fmt()
    .with_target(false)
    .without_time()
    .event_format(CustomFormatter)
    .with_env_filter(EnvFilter::new(format!(
      "dns_adblock={}",
      if verbose { "debug" } else { "info" }
    )))
    .init();
}
