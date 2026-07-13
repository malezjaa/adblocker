use anyhow::{Result, anyhow};
use vox::database::devices::DeviceType;
use vox_shared::pretty::{print_field, print_separator, print_success, print_warning};
use yansi::Paint;

use crate::CliContext;

pub fn pretty_device_type(ty: DeviceType) -> String {
  match ty {
    DeviceType::Windows => "Windows".rgb(56, 189, 248).to_string(),
    DeviceType::Linux => "Linux".rgb(251, 191, 36).to_string(),
    DeviceType::MacOs => "macOS".rgb(167, 139, 250).to_string(),
    DeviceType::Android => "Android".rgb(52, 211, 153).to_string(),
    DeviceType::Ios => "iOS".rgb(251, 113, 133).to_string(),
    DeviceType::Router => "Router".rgb(251, 146, 60).to_string(),
    DeviceType::Other => "Other".rgb(148, 163, 184).to_string(),
  }
}

fn format_last_seen(last_seen: i64) -> (String, bool) {
  let now = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_millis() as i64;

  let diff = now - last_seen;
  let mins = diff / 60_000;
  let hours = diff / 3_600_000;
  let days = diff / 86_400_000;

  if mins < 1 {
    ("Just now".to_string(), true)
  } else if mins < 60 {
    (format!("{}m ago", mins), mins < 5)
  } else if hours < 24 {
    (format!("{}h ago", hours), false)
  } else {
    (format!("{}d ago", days), false)
  }
}

impl CliContext {
  pub async fn new_device(&self, name: String, ty: DeviceType) -> Result<()> {
    let registration = self
      .db
      ._create_device(&name, ty)
      .await
      .map_err(|err| anyhow!("couldn't create a new device: {err}"))?;

    let type_str = pretty_device_type(ty);

    print_success(if registration.restored {
      "Device restored"
    } else {
      "Device created"
    });
    print_separator(30);
    print_field("Name:", name.trim().bold());
    print_field("ID:  ", registration.id.cyan());
    print_field("Type:", type_str);
    print_separator(30);

    Ok(())
  }

  pub async fn remove_device(&self, identifier: String) -> Result<()> {
    let device = self
      .db
      .get_device_by_identifier(&identifier)
      .await
      .map_err(|err| anyhow!("couldn't find device: {err}"))?;

    self
      .db
      .delete_device(&device.id)
      .await
      .map_err(|err| anyhow!("couldn't remove device: {err}"))?;

    print_success("Device removed");
    print_separator(44);
    print_field("Name:", device.name.bold());
    print_field("ID:  ", device.id.cyan());
    print_warning("Adding the same name later will restore this device and ID");
    print_separator(44);

    Ok(())
  }

  pub async fn list_devices(&self) -> Result<()> {
    let devices = self
      .db
      .get_devices()
      .await
      .map_err(|err| anyhow!("couldn't get list of devices: {err}"))?;

    if devices.is_empty() {
      println!();
      println!("  {}", "No devices found.".dim());
      println!();
      return Ok(());
    }

    println!();
    println!(
      "  {} {}",
      "●".dim(),
      format!("{} device{}", devices.len(), if devices.len() == 1 { "" } else { "s" })
        .dim()
    );
    print_separator(44);

    for device in &devices {
      let (last_seen_label, is_recent) = format_last_seen(device.last_seen);

      let wifi = if is_recent {
        "▲".rgb(52, 211, 153).to_string()
      } else {
        "▲".rgb(100, 116, 139).to_string()
      };

      println!(
        "  {}  {}  {}  {}",
        wifi,
        device.name.bold(),
        device.id.cyan().dim(),
        pretty_device_type(device.device_type),
      );
      println!("      {}", last_seen_label.dim());
    }

    print_separator(44);

    Ok(())
  }
}
