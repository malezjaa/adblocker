use std::{fmt, fs::OpenOptions, path::PathBuf};

use tracing::{Event, Level, Subscriber};
use tracing_subscriber::{
  EnvFilter,
  fmt::{FmtContext, FormatEvent, FormatFields},
  layer::SubscriberExt,
  registry::LookupSpan,
  util::SubscriberInitExt,
};
use yansi::Paint;

use crate::logs_dir;

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
      Level::ERROR => Paint::bright_red("[error]").bold(),
      Level::WARN => Paint::bright_yellow("[warning]").bold(),
      Level::INFO => Paint::bright_magenta("[info]").bold(),
      Level::DEBUG => Paint::bright_blue("[debug]").bold(),
      Level::TRACE => Paint::dim("[trace]").bold(),
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
      "vox={0},cli={0},daemon={0},vox_shared={0},vox_windows_client={0}",
      if verbose { "debug" } else { "info" }
    )))
    .init();
}

pub fn setup_service_logger(verbose: bool, component: &str) -> anyhow::Result<PathBuf> {
  let log_dir = logs_dir();
  fs_err::create_dir_all(&log_dir)?;

  let log_path = log_dir.join(format!("{component}.log"));
  let writer_path = log_path.clone();

  let default_level = if verbose { "debug" } else { "info" };

  tracing_subscriber::registry()
    .with(
      tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_ansi(false)
        .event_format(CustomFormatter)
        .with_writer(move || {
          OpenOptions::new()
            .create(true)
            .append(true)
            .open(&writer_path)
            .expect("opening the Vox service log")
        }),
    )
    .with(EnvFilter::new(format!(
      "vox={0},cli={0},daemon={0},vox_shared={0},vox_windows_client={0}",
      if verbose { "debug" } else { "info" }
    )))
    .try_init()
    .map_err(|error| anyhow::anyhow!("failed to initialize logger: {error}"))?;

  tracing::info!(
    log_path = %log_path.display(),
    "service logger initialized"
  );

  Ok(log_path)
}
