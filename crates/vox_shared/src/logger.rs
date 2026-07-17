use std::{
  fmt,
  fs::OpenOptions,
  io::Write,
  path::{Path, PathBuf},
};

use flate2::{Compression, write::GzEncoder};
use time::{
  OffsetDateTime, format_description::BorrowedFormatItem, macros::format_description,
};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::{
  EnvFilter,
  fmt::{
    FmtContext, FormatEvent, FormatFields,
    time::{FormatTime, LocalTime},
  },
  registry::LookupSpan,
  util::SubscriberInitExt,
};
use yansi::{Paint, Style};

use crate::logs_dir;

struct CustomFormatter {
  timer: LocalTime<&'static [BorrowedFormatItem<'static>]>,
}

impl Default for CustomFormatter {
  fn default() -> Self {
    Self {
      timer: LocalTime::new(format_description!(
        "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]"
      )),
    }
  }
}

const TIME_STYLE: Style = Style::new().dim();

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
    let ansi_enabled = writer.has_ansi_escapes();
    if ansi_enabled {
      TIME_STYLE.fmt_prefix(&mut writer)?;
    }
    self.timer.format_time(&mut writer)?;
    if ansi_enabled {
      TIME_STYLE.fmt_suffix(&mut writer)?;
    }

    write!(writer, " ")?;

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
    .event_format(CustomFormatter::default())
    .with_env_filter(EnvFilter::new(format!(
      "vox={0},cli={0},daemon={0},vox_shared={0},vox_windows_client={0}",
      if verbose { "debug" } else { "info" }
    )))
    .init();
}

const MAX_ARCHIVED_LOGS: usize = 5;

fn rotate_log(log_dir: &Path) -> std::io::Result<()> {
  let latest_path = log_dir.join("latest.log");
  if !latest_path.exists() {
    return Ok(());
  }

  let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
  let date_str = format!("{:04}-{:02}-{:02}", now.year(), now.month() as u8, now.day());

  let mut index = 1;
  let archive_path = loop {
    let candidate = log_dir.join(format!("{date_str}-{index}.log.gz"));
    if !candidate.exists() {
      break candidate;
    }
    index += 1;
  };

  let data = fs_err::read(&latest_path)?;
  let file = fs_err::File::create(&archive_path)?;
  let mut encoder = GzEncoder::new(file, Compression::default());
  encoder.write_all(&data)?;
  encoder.finish()?;
  fs_err::remove_file(&latest_path)?;

  prune_old_logs(log_dir)?;
  Ok(())
}

fn prune_old_logs(log_dir: &Path) -> std::io::Result<()> {
  let mut archives: Vec<(PathBuf, std::time::SystemTime)> = fs_err::read_dir(log_dir)?
    .filter_map(|e| e.ok())
    .map(|e| e.path())
    .filter(|p| {
      p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.ends_with(".log.gz"))
    })
    .filter_map(|p| Some((p.clone(), fs_err::metadata(&p).ok()?.modified().ok()?)))
    .collect();

  archives.sort_by_key(|(_, modified)| *modified);
  while archives.len() > MAX_ARCHIVED_LOGS {
    let (oldest, _) = archives.remove(0);
    let _ = fs_err::remove_file(oldest);
  }
  Ok(())
}

pub fn setup_service_logger(verbose: bool, component: &str) {
  let log_dir = logs_dir().join(component);
  fs_err::create_dir_all(&log_dir).expect("creating the Vox log directory");

  rotate_log(&log_dir).expect("rotating previous Vox log");

  let log_path = log_dir.join("latest.log");

  tracing_subscriber::fmt()
    .with_target(false)
    .with_ansi(false)
    .event_format(CustomFormatter::default())
    .with_writer(move || {
      OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .expect("opening the Vox service log")
    })
    .with_env_filter(EnvFilter::new(format!(
      "vox={0},cli={0},daemon={0},vox_shared={0},vox_windows_client={0}",
      if verbose { "debug" } else { "info" }
    )))
    .init();
}
